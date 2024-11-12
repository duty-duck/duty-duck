use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use url::Url;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission}, entity_metadata::EntityMetadata, http_monitor::HttpMonitorStatus
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
    pub metadata: EntityMetadata,
    pub email_notification_enabled: bool,
    pub push_notification_enabled: bool,
    pub sms_notification_enabled: bool,
}

#[derive(Serialize, TS, Clone, Debug)]
#[ts(export)]
pub struct CreateHttpMonitorResponse {
    pub id: Uuid,
}

#[derive(Error, Debug)]
pub enum CreateHttpMonitorError {
    #[error("Failed to create a monitor: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
    #[error("Current user doesn't have the privilege the create HTTP monitors")]
    Forbidden,
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
}

pub async fn create_http_monitor(
    auth_context: &AuthContext,
    repository: &impl HttpMonitorRepository,
    command: CreateHttpMonitorCommand,
) -> Result<CreateHttpMonitorResponse, CreateHttpMonitorError> {
    if !auth_context.can(Permission::WriteHttpMonitors) {
        return Err(CreateHttpMonitorError::Forbidden);
    }

    // Validate URL
    let url = Url::parse(&command.url)?;

    let new_monitor = NewHttpMonitor {
        organization_id: auth_context.active_organization_id,
        url: url.to_string(),
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
        metadata: command.metadata,
        downtime_confirmation_threshold: command.downtime_confirmation_threshold,
        recovery_confirmation_threshold: command.recovery_confirmation_threshold,
        email_notification_enabled: command.email_notification_enabled,
        push_notification_enabled: command.push_notification_enabled,
        sms_notification_enabled: command.sms_notification_enabled,
    };
    let id = repository.create_http_monitor(new_monitor).await?;
    Ok(CreateHttpMonitorResponse { id })
}
