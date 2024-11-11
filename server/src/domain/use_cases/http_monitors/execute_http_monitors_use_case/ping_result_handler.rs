use anyhow::Context;
use chrono::Utc;

use crate::domain::{
    entities::{
        http_monitor::{HttpMonitor, HttpMonitorStatus},
        incident::{
            Incident, IncidentCause, IncidentPriority, IncidentSource, IncidentStatus, NewIncident,
        },
        incident_notification::IncidentNotificationPayload,
    },
    ports::{
        http_monitor_repository::{HttpMonitorRepository, UpdateHttpMonitorStatusCommand},
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::{IncidentRepository, ListIncidentsOpts},
    },
    use_cases::incidents::{confirm_incidents, create_incident, resolve_incidents, NotificationOpts},
};

use super::status_machine;

pub async fn handle_ping_response<HMR, IR, IER, INR>(
    transaction: &mut HMR::Transaction,
    http_monitor_repository: &HMR,
    incident_repository: &IR,
    incident_event_repository: &IER,
    incident_notification_repository: &INR,
    mut monitor: HttpMonitor,
    ping_response: crate::domain::ports::http_client::PingResponse,
) -> anyhow::Result<()>
where
    HMR: HttpMonitorRepository,
    IR: IncidentRepository<Transaction = HMR::Transaction>,
    IER: IncidentEventRepository<Transaction = HMR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = HMR::Transaction>,
{
    let existing_incident =
        get_existing_incident_for_monitor(transaction, incident_repository, &monitor).await?;

    let (status_counter, status) = status_machine::next_status(
        monitor.downtime_confirmation_threshold,
        monitor.recovery_confirmation_threshold,
        monitor.status,
        monitor.status_counter,
        ping_response.error_kind
            == crate::domain::entities::http_monitor::HttpMonitorErrorKind::None,
    );

    let error_kind = ping_response.error_kind;
    let last_http_code = ping_response.http_code.map(|c| c as i16);
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
        .update_http_monitor_status(transaction, patch)
        .await
        .with_context(|| "Failed to update HTTP monitor status")?;

    // Create an unconfirmed incident if the monitor is suspicious and there is no ongoing incident
    if status == HttpMonitorStatus::Suspicious && existing_incident.is_none() {
        create_incident_for_monitor(
            transaction,
            incident_repository,
            incident_event_repository,
            incident_notification_repository,
            &monitor,
            false,
        )
        .await?;
    }

    // Create a new confirmed incident if the monitor is down and there is no ongoing incident
    else if status == HttpMonitorStatus::Down && existing_incident.is_none() {
        create_incident_for_monitor(
            transaction,
            incident_repository,
            incident_event_repository,
            incident_notification_repository,
            &monitor,
            true,
        )
        .await
        .with_context(|| "Failed to create incident for failed HTTP monitor")?;
    }

    // Confirm the incident if the monitor is down and there is an ongoing incident
    else if status == HttpMonitorStatus::Down && existing_incident.is_some() {
        confirm_incident_for_monitor(
            transaction,
            incident_repository,
            incident_event_repository,
            incident_notification_repository,
            &monitor,
        )
        .await?;
    }

    // Resolve the existing incidents if the monitor is up
    else if status == HttpMonitorStatus::Up {
        resolve_incidents_for_monitor(
            transaction,
            incident_repository,
            incident_event_repository,
            incident_notification_repository,
            &monitor,
        )
        .await
        .with_context(|| "Failed to resolve incidents for recovered HTTP monitor")?;
    }

    Ok(())
}

/// Resolves all the incidents for the given monitor
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

/// Returns the existing ongoing incident for the given monitor
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
                include_statuses: &[IncidentStatus::Ongoing, IncidentStatus::ToBeConfirmed],
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
    confirmed_incident: bool,
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    let new_incident = NewIncident {
        organization_id: monitor.organization_id,
        created_by: None,
        status: if confirmed_incident {
            IncidentStatus::Ongoing
        } else {
            IncidentStatus::ToBeConfirmed
        },
        // TODO: let users configure this
        priority: IncidentPriority::Major,
        source: IncidentSource::HttpMonitor { id: monitor.id },
        cause: Some(IncidentCause::HttpMonitorIncidentCause {
            error_kind: monitor.error_kind,
            http_code: monitor.last_http_code,
        }),
        metadata: monitor.metadata.clone(),
    };

    let notification = if confirmed_incident {
        Some(NotificationOpts {
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
        })
    } else {
        None
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

/// Confirms an incident for the given monitor
/// The incident is confirmed in the same transaction as the monitor update.
async fn confirm_incident_for_monitor<IR, IER, INR>(
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

    let notification = Some(NotificationOpts {
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
    });

    confirm_incidents(
        transaction,
        incident_repo,
        incident_event_repo,
        incident_notification_repo,
        monitor.organization_id,
        &[IncidentSource::HttpMonitor { id: monitor.id }],
        notification,
    )
    .await
}
