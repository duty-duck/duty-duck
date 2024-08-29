use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    entities::authorization::AuthContext, ports::user_devices_repository::UserDevicesRepository,
};

#[derive(Debug, Error)]
pub enum RemoveUserDeviceError {
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Failed to register user device: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn remove_user_device(
    auth_context: &AuthContext,
    repository: &impl UserDevicesRepository,
    user_device_id: Uuid,
) -> Result<(), RemoveUserDeviceError> {
    match repository
        .remove_device(auth_context.active_organization_id, user_device_id)
        .await
    {
        Ok(true) => Ok(()),
        Ok(false) => Err(RemoveUserDeviceError::DeviceNotFound),
        Err(e) => Err(RemoveUserDeviceError::TechnicalFailure(e)),
    }
}
