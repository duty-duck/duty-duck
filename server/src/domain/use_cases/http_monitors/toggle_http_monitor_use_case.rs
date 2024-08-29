use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        http_monitor::HttpMonitorStatus,
        incident::IncidentSource,
    },
    ports::{
        http_monitor_repository::{HttpMonitorRepository, UpdateHttpMonitorStatusCommand},
        incident_repository::IncidentRepository,
    },
};

#[derive(Error, Debug)]
pub enum ToggleMonitorError {
    #[error("User has no permission to read this monitor")]
    Forbidden,
    #[error("Monitor not found")]
    NotFound,
    #[error("Failed to toggle monitor: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn toggle_http_monitor<HMR, IR>(
    auth_context: &AuthContext,
    http_monitor_repository: &HMR,
    incident_repository: &IR,
    monitor_id: Uuid,
) -> Result<(), ToggleMonitorError>
where
    HMR: HttpMonitorRepository,
    IR: IncidentRepository<Transaction = HMR::Transaction>,
{
    if !auth_context.can(Permission::WriteHttpMonitors) {
        return Err(ToggleMonitorError::Forbidden);
    }
    let mut tx = http_monitor_repository.begin_transaction().await?;

    let monitor = match http_monitor_repository
        .get_http_monitor(&mut tx, auth_context.active_organization_id, monitor_id)
        .await
    {
        Ok(Some(monitor)) => Ok(monitor),
        Ok(None) => Err(ToggleMonitorError::NotFound),
        Err(e) => Err(ToggleMonitorError::TechnicalFailure(e)),
    }?;

    let now = Utc::now();
    let sources = [IncidentSource::HttpMonitor { id: monitor_id }];
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
            },
        )
        .await?;

    // Resolve any incident previously associated with this monitor
    incident_repository
        .resolve_incidents_by_source(&mut tx, auth_context.active_organization_id, &sources)
        .await?;

    incident_repository.commit_transaction(tx).await?;
    Ok(())
}
