use serde::Deserialize;
use thiserror::Error;
use ts_rs::TS;

use crate::domain::{
    entities::{authorization::AuthContext, user_device::UserDeviceType},
    ports::user_devices_repository::{NewUserDevice, UserDevicesRepository},
};

#[derive(Deserialize, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RegisterUserDeviceCommand {
    pub label: String,
    pub push_notification_token: Option<String>,
    pub device_type: UserDeviceType,
}

#[derive(Debug, Error)]
pub enum RegisterUserDeviceError {
    #[error("Failed to register user device: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn register_user_device(
    auth_context: &AuthContext,
    repository: &impl UserDevicesRepository,
    command: RegisterUserDeviceCommand,
) -> Result<(), RegisterUserDeviceError> {
    let new_device = NewUserDevice {
        organization_id: auth_context.active_organization_id,
        user_id: auth_context.active_user_id,
        label: command.label,
        push_notification_token: command.push_notification_token,
        device_type: command.device_type,
    };
    repository
        .register_device(new_device)
        .await
        .map_err(|e| RegisterUserDeviceError::TechnicalFailure(e))?;
    Ok(())
}
