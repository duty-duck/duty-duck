use std::time::Duration;

use chrono::Utc;
use futures::{stream, StreamExt};
use tokio::task::JoinSet;
use tracing::{debug, error};

use crate::domain::{
    entities::http_monitor::{HttpMonitorErrorKind, HttpMonitorStatus},
    ports::{
        http_client::{HttpClient, PingError},
        http_monitor_repository::{HttpMonitorRepository, UpdateHttpMonitorStatus},
    },
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const DELAY_BETWEEN_TWO_REQUESTS: Duration = Duration::from_secs(1);

pub fn spawn_http_monitors_execution_tasks(
    n_tasks: usize,
    http_monitor_repository: impl HttpMonitorRepository,
    http_client: impl HttpClient,
    select_limit: u32,
    ping_concurrency_limit: usize,
) -> JoinSet<()> {
    let mut join_set = JoinSet::new();
    for _ in 0..n_tasks {
        let repo = http_monitor_repository.clone();
        let http_client = http_client.clone();
        join_set.spawn(async move {
            loop {
                match fetch_and_execute_due_http_monitors(
                    &repo,
                    &http_client,
                    select_limit,
                    REQUEST_TIMEOUT,
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
    http_monitor_repository: &impl HttpMonitorRepository,
    http_client: &impl HttpClient,
    limit: u32,
    request_timeout: Duration,
    concurrency_limit: usize,
) -> anyhow::Result<usize> {
    let mut transaction = http_monitor_repository.begin_transaction().await?;
    let due_monitors = http_monitor_repository
        .list_due_http_monitors(&mut transaction, limit)
        .await?;
    let monitors_len = due_monitors.len();

    let mut ping_results = stream::iter(due_monitors)
        .map(|monitor| {
            let url = monitor.url.clone();
            let http_client = http_client.clone();
            async move { (monitor, http_client.ping(&url, request_timeout).await) }
        })
        .buffer_unordered(concurrency_limit);

    while let Some((monitor, ping_result)) = ping_results.next().await {
        // TODO: implement transitions from/to recovering and suspicious states
        let status = match &ping_result {
            Ok(_) => HttpMonitorStatus::Up,
            Err(_) => HttpMonitorStatus::Down,
        };
        let error_kind = match &ping_result {
            Ok(_) => HttpMonitorErrorKind::None,
            Err(PingError { error_kind, .. }) => *error_kind,
        };
        // TODO: implement status counter
        let status_counter = 0;
        let next_ping_at = Some(Utc::now() + monitor.interval());
        let patch = UpdateHttpMonitorStatus {
            organization_id: monitor.organization_id,
            monitor_id: monitor.id,
            status,
            status_counter,
            next_ping_at,
            error_kind,
        };
        http_monitor_repository
            .update_http_monitor_status(&mut transaction, patch)
            .await?;
    }
    http_monitor_repository
        .commit_transaction(transaction)
        .await?;
    Ok(monitors_len)
}
