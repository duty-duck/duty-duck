use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

use crate::domain::{
    entities::{authorization::AuthContext, user_device::UserDevice},
    ports::user_devices_repository::UserDevicesRepository,
};

#[derive(Debug, Error)]
pub enum ListUserDevicesError {
    #[error("Failed to list user devices: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ListUserDevicesResponse {
    pub devices: Vec<UserDevice>
}

pub async fn list_user_devices(
    auth_context: &AuthContext,
    repository: &impl UserDevicesRepository,
) -> Result<ListUserDevicesResponse, ListUserDevicesError> {
    match repository.list_user_devices(auth_context.active_organization_id, auth_context.active_user_id).await {
        Ok(devices) => Ok(ListUserDevicesResponse { devices }),
        Err(e) => Err(ListUserDevicesError::TechnicalFailure(e)),
    }
}
