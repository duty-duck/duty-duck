use std::time::Duration;

use anyhow::Context;
use chrono::Utc;
use futures::{stream, StreamExt};
use tokio::task::JoinSet;
use tracing::{debug, error};

use crate::domain::{
    entities::{
        http_monitor::{HttpMonitor, HttpMonitorErrorKind, HttpMonitorStatus},
        incident::{IncidentCause, IncidentPriority, IncidentSource, IncidentStatus, NewIncident},
    },
    ports::{
        http_client::{HttpClient, PingError},
        http_monitor_repository::{HttpMonitorRepository, UpdateHttpMonitorStatusCommand},
        incident_repository::IncidentRepository,
    },
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const DELAY_BETWEEN_TWO_REQUESTS: Duration = Duration::from_secs(1);

pub fn spawn_http_monitors_execution_tasks<HMR, IR, HC>(
    n_tasks: usize,
    http_monitor_repository: HMR,
    incident_repository: IR,
    http_client: HC,
    select_limit: u32,
    ping_concurrency_limit: usize,
) -> JoinSet<()>
where
    HMR: HttpMonitorRepository,
    IR: IncidentRepository<Transaction = HMR::Transaction>,
    HC: HttpClient,
{
    let mut join_set = JoinSet::new();
    for _ in 0..n_tasks {
        let http_monitor_repo = http_monitor_repository.clone();
        let incident_repo = incident_repository.clone();
        let http_client = http_client.clone();
        join_set.spawn(async move {
            loop {
                match fetch_and_execute_due_http_monitors(
                    &http_monitor_repo,
                    &incident_repo,
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

async fn fetch_and_execute_due_http_monitors<HMR, IR, HC>(
    http_monitor_repository: &HMR,
    incident_repository: &IR,
    http_client: &HC,
    limit: u32,
    request_timeout: Duration,
    concurrency_limit: usize,
) -> anyhow::Result<usize>
where
    HMR: HttpMonitorRepository,
    IR: IncidentRepository<Transaction = HMR::Transaction>,
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
            async move { (monitor, http_client.ping(&url, request_timeout).await) }
        })
        .buffer_unordered(concurrency_limit);

    while let Some((mut monitor, ping_result)) = ping_results.next().await {
        let (status_counter, status) =
            next_status(monitor.status, monitor.status_counter, ping_result.is_ok());

        let last_http_code = match &ping_result {
            Ok(p) => Some(p.http_code as i16),
            Err(e) => e.http_code.map(|c| c as i16),
        };
        let error_kind = match &ping_result {
            Ok(_) => HttpMonitorErrorKind::None,
            Err(PingError { error_kind, .. }) => *error_kind,
        };
        let next_ping_at = Some(Utc::now() + monitor.interval());
        let last_status_change_at = if status != monitor.status {
            Utc::now()
        } else {
            monitor.last_status_change_at
        };
        let patch = UpdateHttpMonitorStatusCommand {
            organization_id: monitor.organization_id,
            monitor_id: monitor.id,
            last_http_code,
            status,
            status_counter,
            next_ping_at,
            error_kind,
            last_status_change_at,
        };

        // Update the monitor so these info will be used to create the incident
        monitor.error_kind = error_kind;
        monitor.last_http_code = last_http_code;

        // Update the monitor
        http_monitor_repository
            .update_http_monitor_status(&mut transaction, patch)
            .await
            .with_context(|| "Failed to update HTTP monitor status")?;

        // Create a new incident if the monitor becomes down (in the same transaction)
        if status == HttpMonitorStatus::Down
            && monitor.status != HttpMonitorStatus::Down
            && monitor.status != HttpMonitorStatus::Suspicious
        {
            create_incident_for_monitor(&mut transaction, incident_repository, &monitor)
                .await
                .with_context(|| "Failed to create incident for failed HTTP monitor")?;
        }
        // Resolve the existing incidents if the monitor becomes up (in the same transaction)
        else if status == HttpMonitorStatus::Up && monitor.status != HttpMonitorStatus::Up {
            resolve_incidents_for_monitor(&mut transaction, incident_repository, &monitor)
                .await
                .with_context(|| "Failed to resolve incidents for recovered HTTP monitor")?;
        }
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

async fn resolve_incidents_for_monitor<IR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    monitor: &HttpMonitor,
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
{
    incident_repo
        .resolve_incidents_by_source(
            transaction,
            monitor.organization_id,
            &[IncidentSource::HttpMonitor { id: monitor.id }],
        )
        .await
}

async fn create_incident_for_monitor<IR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    monitor: &HttpMonitor,
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
{
    let new_incident = NewIncident {
        organization_id: monitor.organization_id,
        created_by: None,
        status: IncidentStatus::Ongoing,
        // TODO: let users configure this
        priority: IncidentPriority::Major,
        sources: vec![IncidentSource::HttpMonitor { id: monitor.id }],
        cause: Some(IncidentCause::HttpMonitorIncidentCause {
            error_kind: monitor.error_kind,
            http_code: monitor.last_http_code,
        }),
    };

    incident_repo
        .create_incident(transaction, new_incident)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::domain::entities::http_monitor::HttpMonitorStatus;

    use super::next_status;

    #[test]
    fn next_status_tests() {
        // Transition from unknown to up or down
        assert_eq!(
            next_status(HttpMonitorStatus::Unknown, 0, true),
            (0, HttpMonitorStatus::Up)
        );
        assert_eq!(
            next_status(HttpMonitorStatus::Unknown, 0, false),
            (0, HttpMonitorStatus::Down)
        );

        // Down counter increment
        assert_eq!(
            next_status(HttpMonitorStatus::Down, 2, false),
            (3, HttpMonitorStatus::Down)
        );
        assert_eq!(
            next_status(HttpMonitorStatus::Down, 3, false),
            (4, HttpMonitorStatus::Down)
        );

        // Transition from down to recvoering
        assert_eq!(
            next_status(HttpMonitorStatus::Down, 3, true),
            (0, HttpMonitorStatus::Recovering)
        );

        // Recovering counter increment
        assert_eq!(
            next_status(HttpMonitorStatus::Recovering, 0, true),
            (1, HttpMonitorStatus::Recovering)
        );

        // Transition from recovering to up
        assert_eq!(
            next_status(HttpMonitorStatus::Recovering, 2, true),
            (0, HttpMonitorStatus::Up)
        );

        // Up counter increment
        assert_eq!(
            next_status(HttpMonitorStatus::Up, 2, true),
            (3, HttpMonitorStatus::Up)
        );
        assert_eq!(
            next_status(HttpMonitorStatus::Up, 3, true),
            (4, HttpMonitorStatus::Up)
        );

        // Transition from up to suspicious
        assert_eq!(
            next_status(HttpMonitorStatus::Up, 3, false),
            (0, HttpMonitorStatus::Suspicious)
        );

        // Suspicious counter increment
        assert_eq!(
            next_status(HttpMonitorStatus::Suspicious, 0, false),
            (1, HttpMonitorStatus::Suspicious)
        );

        // Transition from suspicious to down
        assert_eq!(
            next_status(HttpMonitorStatus::Suspicious, 1, false),
            (0, HttpMonitorStatus::Down)
        );
    }
}
