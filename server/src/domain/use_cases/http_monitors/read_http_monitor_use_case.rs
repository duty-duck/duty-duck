use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        http_monitor::HttpMonitor,
        incident::{IncidentPriority, IncidentSource, IncidentStatus, IncidentWithSources},
    },
    ports::{
        http_monitor_repository::HttpMonitorRepository, incident_repository::IncidentRepository,
    },
};

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReadHttpMonitorResponse {
    pub monitor: HttpMonitor,
    pub ongoing_incident: Option<IncidentWithSources>,
}

#[derive(Error, Debug)]
pub enum ReadHttpMonitorError {
    #[error("User has no permission to read this monitor")]
    Forbidden,
    #[error("Monitor not found")]
    NotFound,
    #[error("Failed to get monitors from the database: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn read_http_monitor<HMR, IR>(
    auth_context: &AuthContext,
    http_monitor_repository: &HMR,
    incident_repository: &IR,
    monitor_id: Uuid,
) -> Result<ReadHttpMonitorResponse, ReadHttpMonitorError>
where
    HMR: HttpMonitorRepository,
    IR: IncidentRepository<Transaction = HMR::Transaction>,
{
    if !auth_context.can(Permission::ReadHttpMonitors) {
        return Err(ReadHttpMonitorError::Forbidden);
    }
    let mut tx = http_monitor_repository.begin_transaction().await?;

    let monitor = match http_monitor_repository
        .get_http_monitor(&mut tx, auth_context.active_organization_id, monitor_id)
        .await
    {
        Ok(Some(monitor)) => Ok(monitor),
        Ok(None) => Err(ReadHttpMonitorError::NotFound),
        Err(e) => Err(ReadHttpMonitorError::TechnicalFailure(e)),
    }?;

    let sources = [IncidentSource::HttpMonitor { id: monitor_id }];
    let ongoing_incident = incident_repository
        .list_incidents(
            &mut tx,
            auth_context.active_organization_id,
            &[IncidentStatus::Ongoing],
            &IncidentPriority::ALL,
            &sources,
            1,
            0,
        )
        .await?
        .incidents
        .into_iter()
        .next();

    Ok(ReadHttpMonitorResponse {
        monitor,
        ongoing_incident,
    })
}
