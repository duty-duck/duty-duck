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
        let (status_counter, status) =
            next_status(monitor.status, monitor.status_counter, ping_result.is_ok());
        let error_kind = match &ping_result {
            Ok(_) => HttpMonitorErrorKind::None,
            Err(PingError { error_kind, .. }) => *error_kind,
        };
        let next_ping_at = Some(Utc::now() + monitor.interval());
        let patch = UpdateHttpMonitorStatus {
            organization_id: monitor.organization_id,
            monitor_id: monitor.id,
            last_http_code: ping_result.as_ref().ok().map(|r| r.http_code as i16),
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

fn next_status(
    current_status: HttpMonitorStatus,
    current_status_counter: i16,
    last_ping_ok: bool,
) -> (i16, HttpMonitorStatus) {
    let status_confirmation_threshold = 2;

    match (current_status, last_ping_ok) {
        // Transition from unknown/inactive to up/down
        (HttpMonitorStatus::Unknown | HttpMonitorStatus::Inactive, true) => {
            (0, HttpMonitorStatus::Up)
        }
        (HttpMonitorStatus::Unknown | HttpMonitorStatus::Inactive, false) => {
            (0, HttpMonitorStatus::Down)
        }
        // Down monitor staying down
        (HttpMonitorStatus::Down, false) => (
            current_status_counter.saturating_add(1),
            HttpMonitorStatus::Down,
        ),
        // Transition from down to recovering
        (HttpMonitorStatus::Down, true) => (0, HttpMonitorStatus::Recovering),
        // Transition from suspicious to down
        (HttpMonitorStatus::Suspicious, false) => {
            let next_status_counter = current_status_counter.saturating_add(1);
            if next_status_counter >= status_confirmation_threshold {
                (0, HttpMonitorStatus::Down)
            } else {
                (next_status_counter, HttpMonitorStatus::Suspicious)
            }
        }
        // Transition from suspicious to recovering
        (HttpMonitorStatus::Suspicious, true) => (0, HttpMonitorStatus::Recovering),
        // Transition from recovering back to suspicious
        (HttpMonitorStatus::Recovering, false) => (0, HttpMonitorStatus::Suspicious),
        // Transition from recovering to up
        (HttpMonitorStatus::Recovering, true) => {
            let next_status_counter = current_status_counter.saturating_add(1);
            if next_status_counter >= status_confirmation_threshold {
                (0, HttpMonitorStatus::Up)
            } else {
                (next_status_counter, HttpMonitorStatus::Recovering)
            }
        }
        // Transition from up to suspicious
        (HttpMonitorStatus::Up, false) => (0, HttpMonitorStatus::Suspicious),
        // Up monitor staying up
        (HttpMonitorStatus::Up, true) => (
            current_status_counter.saturating_add(1),
            HttpMonitorStatus::Up,
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entities::http_monitor::HttpMonitorStatus;

    use super::next_status;

    #[test]
    fn next_status_tests() {
        // Transition from unknown to up or down
        assert_eq!(next_status(HttpMonitorStatus::Unknown, 0, true), (0, HttpMonitorStatus::Up));
        assert_eq!(next_status(HttpMonitorStatus::Unknown, 0, false), (0, HttpMonitorStatus::Down));

        // Down counter increment
        assert_eq!(next_status(HttpMonitorStatus::Down, 2, false), (3, HttpMonitorStatus::Down));
        assert_eq!(next_status(HttpMonitorStatus::Down, 3, false), (4, HttpMonitorStatus::Down));

        // Transition from down to recvoering
        assert_eq!(next_status(HttpMonitorStatus::Down, 3, true), (0, HttpMonitorStatus::Recovering));

        // Recovering counter increment
        assert_eq!(next_status(HttpMonitorStatus::Recovering, 0, true), (1, HttpMonitorStatus::Recovering));
        
        // Transition from recovering to up
        assert_eq!(next_status(HttpMonitorStatus::Recovering, 2, true), (0, HttpMonitorStatus::Up));

        // Up counter increment
        assert_eq!(next_status(HttpMonitorStatus::Up, 2, true), (3, HttpMonitorStatus::Up));
        assert_eq!(next_status(HttpMonitorStatus::Up, 3, true), (4, HttpMonitorStatus::Up));

        // Transition from up to suspicious
        assert_eq!(next_status(HttpMonitorStatus::Up, 3, false), (0, HttpMonitorStatus::Suspicious));

        // Suspicious counter increment
        assert_eq!(next_status(HttpMonitorStatus::Suspicious, 0, false), (1, HttpMonitorStatus::Suspicious));
        
        // Transition from suspicious to down
        assert_eq!(next_status(HttpMonitorStatus::Suspicious, 1, false), (0, HttpMonitorStatus::Down));
   } 
}