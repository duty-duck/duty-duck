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

#[allow(clippy::too_many_arguments)]
pub fn spawn_http_monitors_execution_tasks<HMR, IR, IER, INR, HC>(
    n_tasks: usize,
    http_monitor_repository: HMR,
    incident_repository: IR,
    incident_event_repository: IER,
    incident_notification_repository: INR,
    http_client: HC,
    select_limit: u32,
    ping_concurrency_limit: usize,
) -> JoinSet<()>
where
    HMR: HttpMonitorRepository,
    IR: IncidentRepository<Transaction = HMR::Transaction>,
    IER: IncidentEventRepository<Transaction = HMR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = HMR::Transaction>,
    HC: HttpClient,
{
    let mut join_set = JoinSet::new();
    for _ in 0..n_tasks {
        let http_monitor_repo = http_monitor_repository.clone();
        let incident_repo = incident_repository.clone();
        let incident_event_repo = incident_event_repository.clone();
        let incident_notification_repository = incident_notification_repository.clone();
        let http_client = http_client.clone();
        join_set.spawn(async move {
            loop {
                match fetch_and_execute_due_http_monitors(
                    &http_monitor_repo,
                    &incident_repo,
                    &incident_event_repo,
                    &incident_notification_repository,
                    &http_client,
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

#[allow(clippy::too_many_arguments)]
async fn fetch_and_execute_due_http_monitors<HMR, IR, IER, INR, HC>(
    http_monitor_repository: &HMR,
    incident_repository: &IR,
    incident_event_repository: &IER,
    incident_notification_repository: &INR,
    http_client: &HC,
    limit: u32,
    concurrency_limit: usize,
) -> anyhow::Result<usize>
where
    HMR: HttpMonitorRepository,
    IR: IncidentRepository<Transaction = HMR::Transaction>,
    IER: IncidentEventRepository<Transaction = HMR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = HMR::Transaction>,
    HC: HttpClient,
{
    let mut transaction = http_monitor_repository.begin_transaction().await?;
    let due_monitors = http_monitor_repository
        .list_due_http_monitors(&mut transaction, limit)
        .await?;
    let monitors_len = due_monitors.len();

    let mut ping_results = stream::iter(due_monitors)
        .map(|monitor| {
            let url = monitor.url.clone();
            let http_client = http_client.clone();
            async move { (monitor, http_client.ping(&url, REQUEST_TIMEOUT).await) }
        })
        .buffer_unordered(concurrency_limit);

    while let Some((monitor, ping_result)) = ping_results.next().await {
        ping_result_handler::handle_ping_response(
            &mut transaction,
            http_monitor_repository,
            incident_repository,
            incident_event_repository,
            incident_notification_repository,
            monitor,
            ping_result,
        )
        .await?;
    }
    http_monitor_repository
        .commit_transaction(transaction)
        .await?;
    Ok(monitors_len)
}
