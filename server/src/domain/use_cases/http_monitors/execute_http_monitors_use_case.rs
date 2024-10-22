use std::time::Duration;

use anyhow::Context;
use chrono::Utc;
use futures::{stream, StreamExt};
use tokio::task::JoinSet;
use tracing::{debug, error};

use crate::domain::{
    entities::{
        http_monitor::{HttpMonitor, HttpMonitorErrorKind, HttpMonitorStatus},
        incident::{
            Incident, IncidentCause, IncidentPriority, IncidentSource, IncidentStatus, NewIncident,
        },
        incident_notification::IncidentNotificationPayload,
    },
    ports::{
        http_client::{HttpClient, PingError},
        http_monitor_repository::{HttpMonitorRepository, UpdateHttpMonitorStatusCommand},
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::{IncidentRepository, ListIncidentsOpts},
    },
    use_cases::incidents::{create_incident, resolve_incidents, NotificationOpts},
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

    while let Some((mut monitor, ping_result)) = ping_results.next().await {
        let (status_counter, status) = next_status(
            monitor.downtime_confirmation_threshold,
            monitor.recovery_confirmation_threshold,
            monitor.status,
            monitor.status_counter,
            ping_result.is_ok(),
        );

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

        // Create a new incident if the monitor is down and there is no ongoing incident
        if status == HttpMonitorStatus::Down
            && get_existing_incident_for_monitor(&mut transaction, incident_repository, &monitor)
                .await?
                .is_none()
        {
            create_incident_for_monitor(
                &mut transaction,
                incident_repository,
                incident_event_repository,
                incident_notification_repository,
                &monitor,
            )
            .await
            .with_context(|| "Failed to create incident for failed HTTP monitor")?;
        }
        // Resolve the existing incidents if the monitor is up
        // Don't bother checking for ongoing incidents as we are going to resolve all of them.
        // This ensures that any incident that wasn't resolved before because of a database error will be resolved.
        else if status == HttpMonitorStatus::Up {
            resolve_incidents_for_monitor(
                &mut transaction,
                incident_repository,
                incident_event_repository,
                incident_notification_repository,
                &monitor,
            )
            .await
            .with_context(|| "Failed to resolve incidents for recovered HTTP monitor")?;
        }
    }
    http_monitor_repository
        .commit_transaction(transaction)
        .await?;
    Ok(monitors_len)
}

/// Determines the next status of an HTTP monitor based on its current state and the latest ping result.
/// Returns a tuple of the new status counter and the new status.
fn next_status(
    downtime_confirmation_threshold: i16,
    recovery_confirmation_threshold: i16,
    current_status: HttpMonitorStatus,
    current_status_counter: i16,
    last_ping_ok: bool,
) -> (i16, HttpMonitorStatus) {
    match (current_status, last_ping_ok) {
        // Transition from unknown/inactive to up/down
        (HttpMonitorStatus::Unknown | HttpMonitorStatus::Inactive, true) => {
            (1, HttpMonitorStatus::Up)
        }
        (HttpMonitorStatus::Unknown | HttpMonitorStatus::Inactive, false) => {
            (1, HttpMonitorStatus::Down)
        }
        // Down monitor staying down
        (HttpMonitorStatus::Down, false) => (
            current_status_counter.saturating_add(1),
            HttpMonitorStatus::Down,
        ),
        // Transition from down to recovering
        (HttpMonitorStatus::Down, true) if recovery_confirmation_threshold > 1 => {
            (1, HttpMonitorStatus::Recovering)
        }
        // Transition from down to up (no confirmation)
        (HttpMonitorStatus::Down, true) => (1, HttpMonitorStatus::Up),
        // Transition from suspicious to down
        (HttpMonitorStatus::Suspicious, false) => {
            let next_status_counter = current_status_counter.saturating_add(1);
            if next_status_counter >= downtime_confirmation_threshold {
                (1, HttpMonitorStatus::Down)
            } else {
                (next_status_counter, HttpMonitorStatus::Suspicious)
            }
        }
        // Transition from suspicious to recovering
        (HttpMonitorStatus::Suspicious, true) => (1, HttpMonitorStatus::Recovering),
        // Transition from recovering back to suspicious
        (HttpMonitorStatus::Recovering, false) if downtime_confirmation_threshold > 1 => {
            (1, HttpMonitorStatus::Suspicious)
        }
        // Transition from recovering to down (no confirmation)
        (HttpMonitorStatus::Recovering, false) => (1, HttpMonitorStatus::Down),
        // Transition from recovering to up
        (HttpMonitorStatus::Recovering, true) => {
            let next_status_counter = current_status_counter.saturating_add(1);
            if next_status_counter >= recovery_confirmation_threshold {
                (1, HttpMonitorStatus::Up)
            } else {
                (next_status_counter, HttpMonitorStatus::Recovering)
            }
        }
        // Transition from up to suspicious
        (HttpMonitorStatus::Up, false) if downtime_confirmation_threshold > 1 => {
            (1, HttpMonitorStatus::Suspicious)
        }
        // Transition from up to down (no confirmation)
        (HttpMonitorStatus::Up, false) => (1, HttpMonitorStatus::Down),

        // Up monitor staying up
        (HttpMonitorStatus::Up, true) => (
            current_status_counter.saturating_add(1),
            HttpMonitorStatus::Up,
        ),
    }
}

async fn resolve_incidents_for_monitor<IR, IER, INR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    monitor: &HttpMonitor,
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    resolve_incidents(
        transaction,
        incident_repo,
        incident_event_repo,
        incident_notification_repo,
        monitor.organization_id,
        &[IncidentSource::HttpMonitor { id: monitor.id }],
    )
    .await
}

async fn get_existing_incident_for_monitor<IR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    monitor: &HttpMonitor,
) -> anyhow::Result<Option<Incident>>
where
    IR: IncidentRepository,
{
    let incident = incident_repo
        .list_incidents(
            transaction,
            monitor.organization_id,
            ListIncidentsOpts {
                include_statuses: &[IncidentStatus::Ongoing],
                include_priorities: &IncidentPriority::ALL,
                include_sources: &[IncidentSource::HttpMonitor { id: monitor.id }],
                limit: 1,
                ..Default::default()
            },
        )
        .await?
        .incidents
        .into_iter()
        .next();
    Ok(incident)
}

/// Creates a new incident for the given monitor
/// The incident is created in the same transaction as the monitor update.
async fn create_incident_for_monitor<IR, IER, INR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    monitor: &HttpMonitor,
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    let new_incident = NewIncident {
        organization_id: monitor.organization_id,
        created_by: None,
        status: IncidentStatus::Ongoing,
        // TODO: let users configure this
        priority: IncidentPriority::Major,
        source: IncidentSource::HttpMonitor { id: monitor.id },
        cause: Some(IncidentCause::HttpMonitorIncidentCause {
            error_kind: monitor.error_kind,
            http_code: monitor.last_http_code,
        }),
    };

    let notification = NotificationOpts {
        send_sms: monitor.sms_notification_enabled,
        send_push_notification: monitor.push_notification_enabled,
        send_email: monitor.email_notification_enabled,
        notification_payload: IncidentNotificationPayload {
            incident_cause: IncidentCause::HttpMonitorIncidentCause {
                error_kind: monitor.error_kind,
                http_code: monitor.last_http_code,
            },
            incident_http_monitor_url: Some(monitor.url.clone()),
        },
    };

    create_incident(
        transaction,
        incident_repo,
        incident_event_repo,
        incident_notification_repo,
        new_incident,
        notification,
    )
    .await
}

#[cfg(test)]
mod tests {
    use crate::domain::entities::http_monitor::HttpMonitorStatus;

    use super::next_status;

    #[test]
    fn next_status_tests_with_confirmation() {
        // Transition from unknown to up or down
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Unknown, 0, true),
            (1, HttpMonitorStatus::Up)
        );
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Unknown, 0, false),
            (1, HttpMonitorStatus::Down)
        );

        // Down counter increment
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Down, 2, false),
            (3, HttpMonitorStatus::Down)
        );
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Down, 3, false),
            (4, HttpMonitorStatus::Down)
        );

        // Transition from down to recovering
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Down, 3, true),
            (1, HttpMonitorStatus::Recovering)
        );

        // Recovering counter increment (confirmation threshold = 2)
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Recovering, 0, true),
            (1, HttpMonitorStatus::Recovering)
        );

        // Transition from recovering to up
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Recovering, 2, true),
            (1, HttpMonitorStatus::Up)
        );

        // Up counter increment
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Up, 2, true),
            (3, HttpMonitorStatus::Up)
        );
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Up, 3, true),
            (4, HttpMonitorStatus::Up)
        );

        // Transition from up to suspicious
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Up, 3, false),
            (1, HttpMonitorStatus::Suspicious)
        );

        // Suspicious counter increment
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Suspicious, 0, false),
            (1, HttpMonitorStatus::Suspicious)
        );

        // Transition from suspicious to down
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Suspicious, 1, false),
            (1, HttpMonitorStatus::Down)
        );
    }

    #[test]
    fn next_status_tests_no_confirmation() {
        // Transition from unknown to up or down
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Unknown, 0, true),
            (1, HttpMonitorStatus::Up)
        );
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Unknown, 0, false),
            (1, HttpMonitorStatus::Down)
        );

        // Down counter increment
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Down, 2, false),
            (3, HttpMonitorStatus::Down)
        );
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Down, 3, false),
            (4, HttpMonitorStatus::Down)
        );

        // Transition from down to up (no confirmation)
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Down, 3, true),
            (1, HttpMonitorStatus::Up)
        );

        // Up counter increment
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Up, 2, true),
            (3, HttpMonitorStatus::Up)
        );
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Up, 3, true),
            (4, HttpMonitorStatus::Up)
        );

        // Transition from up to down
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Up, 3, false),
            (1, HttpMonitorStatus::Down)
        );

        // Transition from suspicious to down
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Suspicious, 0, false),
            (1, HttpMonitorStatus::Down)
        );

        // Transition from recovering to up
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Recovering, 0, true),
            (1, HttpMonitorStatus::Up)
        );
    }
}
