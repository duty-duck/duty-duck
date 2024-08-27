use std::collections::HashSet;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        http_monitor::HttpMonitorStatus,
    },
    ports::http_monitor_repository::{HttpMonitorRepository, NewHttpMonitor},
};

#[derive(Deserialize, TS, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateHttpMonitorCommand {
    pub url: String,
    pub interval_seconds: u32,
    pub recovery_confirmation_threshold: u32,
    pub downtime_confirmation_threshold: u32,
    pub is_active: bool,
    pub tags: HashSet<String>,
}

#[derive(Serialize, TS, Clone, Debug)]
#[ts(export)]
pub struct CreateHttpMonitorResponse {
    pub id: Uuid,
}

#[derive(Error, Debug)]
pub enum CreateHttpMonitorError {
    #[error("Failed to create a monitor: {0}")]
    TechnicalError(#[from] anyhow::Error),
    #[error("Current user doesn't have the privilege the create HTTP monitors")]
    Forbidden,
}

pub async fn create_http_monitor(
    auth_context: &AuthContext,
    repository: &impl HttpMonitorRepository,
    command: CreateHttpMonitorCommand,
) -> Result<CreateHttpMonitorResponse, CreateHttpMonitorError> {
    if !auth_context.can(Permission::WriteHttpMonitors) {
        return Err(CreateHttpMonitorError::Forbidden);
    }

    let new_monitor = NewHttpMonitor {
        organization_id: auth_context.active_organization_id,
        url: command.url,
        status: if command.is_active {
            HttpMonitorStatus::Unknown
        } else {
            HttpMonitorStatus::Inactive
        },
        next_ping_at: if command.is_active {
            Some(Utc::now())
        } else {
            None
        },
        interval_seconds: command.interval_seconds,
        tags: command.tags.into_iter().collect(),
        downtime_confirmation_threshold: command.downtime_confirmation_threshold,
        recovery_confirmation_threshold: command.recovery_confirmation_threshold
    };
    let id = repository.create_http_monitor(new_monitor).await?;
    Ok(CreateHttpMonitorResponse { id })
}
