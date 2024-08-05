use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        http_monitor::{HttpMonitor, HttpMonitorStatus},
    },
    ports::http_monitor_repository::{HttpMonitorRepository, ListHttpMonitorsOutput},
};

#[derive(Error, Debug)]
pub enum ReadHttpMonitorError {
    #[error("User has no permission to read this monitor")]
    Forbidden,
    #[error("Monitor not found")]
    NotFound,
    #[error("Failed to get monitors from the database: {0}")]
    TechnicalError(#[from] anyhow::Error),
}

pub async fn read_http_monitor(
    auth_context: &AuthContext,
    repository: &impl HttpMonitorRepository,
    monitor_id: Uuid,
) -> Result<HttpMonitor, ReadHttpMonitorError> {
    if !auth_context.can(Permission::ReadHttpMonitors) {
        return Err(ReadHttpMonitorError::Forbidden);
    }

    match repository
        .get_http_monitor(auth_context.active_organization_id, monitor_id)
        .await
    {
        Ok(Some(monitor)) => Ok(monitor),
        Ok(None) => Err(ReadHttpMonitorError::NotFound),
        Err(e) => Err(ReadHttpMonitorError::TechnicalError(e)),
    }
}
