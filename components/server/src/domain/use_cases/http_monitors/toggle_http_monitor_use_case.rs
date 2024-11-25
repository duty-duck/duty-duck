use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        http_monitor::HttpMonitorStatus,
        incident::{IncidentPriority, IncidentSource, IncidentStatus},
    },
    ports::{
        http_monitor_repository::{HttpMonitorRepository, UpdateHttpMonitorStatusCommand},
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::{IncidentRepository, ListIncidentsOpts},
    },
    use_cases::incidents::resolve_incident,
};

#[derive(Error, Debug)]
pub enum ToggleMonitorError {
    #[error("User has no permission to read this monitor")]
    Forbidden,
    #[error("Monitor not found")]
    NotFound,
    #[error("Monitor is archived and cannot be toggled")]
    MonitorIsArchived,
    #[error("Failed to toggle monitor: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn toggle_http_monitor<HMR, IR, IER, INR>(
    auth_context: &AuthContext,
    http_monitor_repository: &HMR,
    incident_repository: &IR,
    incident_event_repository: &IER,
    incident_notification_repository: &INR,
    monitor_id: Uuid,
) -> Result<(), ToggleMonitorError>
where
    HMR: HttpMonitorRepository,
    IR: IncidentRepository<Transaction = HMR::Transaction>,
    IER: IncidentEventRepository<Transaction = HMR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = HMR::Transaction>,
{
    if !auth_context.can(Permission::WriteHttpMonitors) {
        return Err(ToggleMonitorError::Forbidden);
    }
    let mut tx = http_monitor_repository.begin_transaction().await?;

    let monitor = match http_monitor_repository
        .get_http_monitor(&mut tx, auth_context.active_organization_id, monitor_id)
        .await
    {
        Ok(Some(monitor)) if monitor.archived_at.is_some() => {
            return Err(ToggleMonitorError::MonitorIsArchived);
        }
        Ok(Some(monitor)) => Ok(monitor),
        Ok(None) => Err(ToggleMonitorError::NotFound),
        Err(e) => Err(ToggleMonitorError::TechnicalFailure(e)),
    }?;

    let now = Utc::now();
    let (status, next_ping_at) = if monitor.status == HttpMonitorStatus::Inactive {
        (HttpMonitorStatus::Unknown, Some(now))
    } else {
        (HttpMonitorStatus::Inactive, None)
    };

    http_monitor_repository
        .update_http_monitor_status(
            &mut tx,
            UpdateHttpMonitorStatusCommand {
                organization_id: auth_context.active_organization_id,
                monitor_id,
                status,
                next_ping_at,
                last_status_change_at: now,
                status_counter: 0,
                error_kind: monitor.error_kind,
                last_http_code: monitor.last_http_code,
                archived_at: None,
            },
        )
        .await?;

    // Retrieve all ongoing incidents for this monitor
    let ongoing_incidents = incident_repository
        .list_incidents(
            &mut tx,
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
        .incidents;

    for incident in ongoing_incidents {
        resolve_incident(
            &mut tx,
            incident_repository,
            incident_event_repository,
            incident_notification_repository,
            &incident,
        )
        .await?;
    }

    incident_repository.commit_transaction(tx).await?;
    Ok(())
}
