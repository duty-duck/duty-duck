use anyhow::Context;
use chrono::Utc;

use crate::domain::{
    entities::{
        http_monitor::{HttpMonitor, HttpMonitorStatus},
        incident::{
            HttpMonitorIncidentCause, Incident, IncidentCause, IncidentPriority, IncidentSource,
            IncidentStatus, NewIncident,
        },
        incident_event::{
            IncidentEvent, IncidentEventPayload, IncidentEventType, PingEventPayload,
        },
        incident_notification::IncidentNotificationPayload,
    },
    ports::{
        http_monitor_repository::{HttpMonitorRepository, UpdateHttpMonitorStatusCommand},
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::{IncidentRepository, ListIncidentsOpts},
    },
    use_cases::incidents::{
        confirm_incident, create_incident, resolve_incident, NotificationOpts,
        ResolveIncidentOutput,
    },
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

    let (incident_cause, cause_has_changed) = build_incident_cause(
        &monitor,
        existing_incident.as_ref().and_then(|i| i.cause.as_ref()),
    );

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
            incident_cause,
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
            incident_cause,
        )
        .await
        .with_context(|| "Failed to create incident for failed HTTP monitor")?;
    }
    // if there is an existing incident ...
    else if let Some(existing_incident) = existing_incident {
        let updated_incident = Incident {
            cause: Some(incident_cause),
            ..existing_incident.clone()
        };

        let existing_incident_has_been_deleted;
        if status == HttpMonitorStatus::Down && existing_incident.status != IncidentStatus::Ongoing
        {
            existing_incident_has_been_deleted = false;
            // if the monitor is down we may need to confirm the incident if it is not confirmed yet
            confirm_incident_for_monitor(
                transaction,
                incident_repository,
                incident_event_repository,
                incident_notification_repository,
                &monitor,
                &updated_incident,
            )
            .await?;
        }
        // else if the is up, we need to resolve the incident
        else if status == HttpMonitorStatus::Up {
            let output = resolve_incident(
                transaction,
                incident_repository,
                incident_event_repository,
                incident_notification_repository,
                &updated_incident,
            )
            .await?;

            existing_incident_has_been_deleted =
                matches!(output, ResolveIncidentOutput::IncidentDeleted);
        } else {
            existing_incident_has_been_deleted = false;
        }

        // if the incident cause has changed, we need to create a ping event
        if cause_has_changed && !existing_incident_has_been_deleted {
            let ping_event = IncidentEvent {
                incident_id: updated_incident.id,
                event_type: IncidentEventType::MonitorPinged,
                event_payload: Some(IncidentEventPayload::MonitorPing(PingEventPayload {
                    error_kind: monitor.error_kind,
                    http_code: monitor.last_http_code.map(|c| c as i32),
                })),
                organization_id: monitor.organization_id,
                user_id: None,
                created_at: Utc::now(),
            };
            incident_event_repository
                .create_incident_event(transaction, ping_event)
                .await?;
        }
    };

    Ok(())
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
/// Returns the id of the created incident
async fn create_incident_for_monitor<IR, IER, INR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    monitor: &HttpMonitor,
    confirmed_incident: bool,
    incident_cause: IncidentCause,
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    let mut metadata = monitor.metadata.clone();
    if let Some(http_code) = monitor.last_http_code {
        metadata
            .records
            .insert("http_code".to_string(), http_code.to_string());
    }
    if let Ok(url) = monitor.url() {
        metadata.records.insert("url".to_string(), url.to_string());
        if let Some(host) = url.host_str() {
            metadata
                .records
                .insert("host".to_string(), host.to_string());
        }
    }

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
        cause: Some(incident_cause.clone()),
        metadata: monitor.metadata.clone(),
    };

    // send a notification only if the incident is confirmed
    let notification = if confirmed_incident {
        Some(NotificationOpts {
            send_sms: monitor.sms_notification_enabled,
            send_push_notification: monitor.push_notification_enabled,
            send_email: monitor.email_notification_enabled,
            notification_payload: IncidentNotificationPayload {
                incident_cause,
                incident_http_monitor_url: Some(monitor.url.clone()),
            },
        })
    } else {
        None
    };

    let incident_id = create_incident(
        transaction,
        incident_repo,
        incident_event_repo,
        incident_notification_repo,
        new_incident,
        notification,
    )
    .await?;

    let ping_event = IncidentEvent {
        incident_id,
        event_type: IncidentEventType::MonitorPinged,
        event_payload: Some(IncidentEventPayload::MonitorPing(PingEventPayload {
            error_kind: monitor.error_kind,
            http_code: monitor.last_http_code.map(|c| c as i32),
        })),
        organization_id: monitor.organization_id,
        user_id: None,
        created_at: Utc::now(),
    };
    incident_event_repo
        .create_incident_event(transaction, ping_event)
        .await?;

    Ok(())
}

/// Confirms an incident for the given monitor
/// The incident is confirmed in the same transaction as the monitor update.
async fn confirm_incident_for_monitor<IR, IER, INR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    monitor: &HttpMonitor,
    incident: &Incident,
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    let notification = NotificationOpts {
        send_sms: monitor.sms_notification_enabled,
        send_push_notification: monitor.push_notification_enabled,
        send_email: monitor.email_notification_enabled,
        notification_payload: IncidentNotificationPayload {
            incident_cause: incident
                .cause
                .clone()
                .context("Incident cause is required")?,
            incident_http_monitor_url: Some(monitor.url.clone()),
        },
    };

    confirm_incident(
        transaction,
        incident_repo,
        incident_event_repo,
        incident_notification_repo,
        incident,
        notification,
    )
    .await
}

/// Builds the incident cause for the given monitor
/// Returns a tuple with the new incident cause and a boolean indicating if the cause has changed
fn build_incident_cause(
    monitor: &HttpMonitor,
    previous_cause: Option<&IncidentCause>,
) -> (IncidentCause, bool) {
    let last_ping = HttpMonitorIncidentCause {
        error_kind: monitor.error_kind,
        http_code: monitor.last_http_code,
    };

    match previous_cause {
        // if there was an existing incident with an HttpMonitorIncidentCause, we may need to update the previous_pings field
        Some(IncidentCause::HttpMonitorIncidentCause {
            last_ping: previous_last_ping,
            previous_pings,
        }) => {
            let mut previous_pings = previous_pings.clone();
            let mut cause_has_changed = false;

            // if the cause has changed, we need to add the previous cause to the previous_pings field
            if previous_last_ping.error_kind != last_ping.error_kind
                || previous_last_ping.http_code != last_ping.http_code
            {
                previous_pings.push(previous_last_ping.clone());
                cause_has_changed = true;
            }

            // limit the number of previous pings to 10
            previous_pings.truncate(10);

            (
                IncidentCause::HttpMonitorIncidentCause {
                    last_ping,
                    previous_pings,
                },
                cause_has_changed,
            )
        }
        _ => (
            IncidentCause::HttpMonitorIncidentCause {
                last_ping,
                previous_pings: Default::default(),
            },
            false,
        ),
    }
}
