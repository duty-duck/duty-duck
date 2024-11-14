use std::time::Duration;

use futures::{stream, StreamExt};
use tokio::task::JoinSet;
use tracing::{debug, error};

mod ping_result_handler;
mod status_machine;

#[cfg(test)]
mod tests;

use crate::domain::ports::{
    http_client::HttpClient, http_monitor_repository::HttpMonitorRepository,
    incident_event_repository::IncidentEventRepository,
    incident_notification_repository::IncidentNotificationRepository,
    incident_repository::IncidentRepository,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const DELAY_BETWEEN_TWO_REQUESTS: Duration = Duration::from_secs(1);

#[derive(Clone)]
pub struct ExecuteHttpMonitorsUseCase<HMR, IR, IER, INR, HC> {
    pub http_monitor_repository: HMR,
    pub incident_repository: IR,
    pub incident_event_repository: IER,
    pub incident_notification_repository: INR,
    pub http_client: HC,
}

impl<HMR, IR, IER, INR, HC> ExecuteHttpMonitorsUseCase<HMR, IR, IER, INR, HC>
where
    HMR: HttpMonitorRepository,
    IR: IncidentRepository<Transaction = HMR::Transaction>,
    IER: IncidentEventRepository<Transaction = HMR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = HMR::Transaction>,
    HC: HttpClient,
{
    /// Spawns a set of tasks that will ping the monitors concurrently and handle the results
    /// Returns a join set of tasks
    /// The tasks will run indefinitely until the application is terminated
    pub fn spawn_http_monitors_execution_tasks(
        self,
        n_tasks: usize,
        select_limit: u32,
        ping_concurrency_limit: usize,
    ) -> JoinSet<()> {
        let mut join_set = JoinSet::new();
        for _ in 0..n_tasks {
            let this = self.clone();
            join_set.spawn(async move {
                loop {
                    match this.fetch_and_execute_due_http_monitors(
                        select_limit,
                        ping_concurrency_limit,
                    )
                    .await
                    {
                        Ok(monitors) if monitors > 0 => {
                            debug!(monitors, "Executed {} monitors", monitors);
                        }
                        Err(e) => {
                            error!(error = ?e, "Failed to execute one or more monitors")
                        }
                        Ok(_) => {}
                    }
                    tokio::time::sleep(DELAY_BETWEEN_TWO_REQUESTS).await;
                }
            });
        }
        join_set
    }

    async fn fetch_and_execute_due_http_monitors(
        &self,
        limit: u32,
        concurrency_limit: usize,
    ) -> anyhow::Result<usize> {
        let mut transaction = self.http_monitor_repository.begin_transaction().await?;

        // Fetch the monitors that are due to be pinged
        let due_monitors = self
            .http_monitor_repository
            .list_due_http_monitors(&mut transaction, limit)
            .await?;
        let monitors_len = due_monitors.len();

        // Ping the monitors concurrently and collect the results
        let mut ping_results = stream::iter(due_monitors)
            .map(|monitor| {
                let url = monitor.url.clone();
                let http_client = self.http_client.clone();
                async move { (monitor, http_client.ping(&url, REQUEST_TIMEOUT).await) }
            })
            .buffer_unordered(concurrency_limit);

        // Go through the ping results and handle them
        while let Some((monitor, ping_result)) = ping_results.next().await {
            let existing_incident =
                self.get_existing_incident_for_monitor(&mut transaction, &monitor)
                    .await?;

            self.handle_ping_response(
                &mut transaction,
                monitor,
                ping_result,
                existing_incident,
            )
            .await?;
        }

        // Finally, commit the transaction to persist the changes
        self.http_monitor_repository
            .commit_transaction(transaction)
            .await?;
        Ok(monitors_len)
    }
}
