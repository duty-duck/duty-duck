use crate::domain::{
    entities::{authorization::AuthContext, push_notification::PushNotification},
    ports::{
        push_notification_server::PushNotificationServer,
        user_devices_repository::UserDevicesRepository,
    },
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SendTestNotificationError {
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Failed to send notification to user device: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn send_test_notification<UDR, PNR>(
    user_devices_repo: &UDR,
    push_notification_server: &PNR,
    auth_context: &AuthContext,
    device_id: Uuid,
) -> Result<(), SendTestNotificationError>
where
    UDR: UserDevicesRepository,
    PNR: PushNotificationServer,
{
    let device = user_devices_repo
        .get_user_device(
            auth_context.active_organization_id()?,
            auth_context.active_user_id,
            device_id,
        )
        .await?
        .ok_or(SendTestNotificationError::DeviceNotFound)?;

    let token = device
        .push_notification_token
        .0
        .ok_or(SendTestNotificationError::DeviceNotFound)?;

    let notification = PushNotification {
        title: t!("testPushNotificationTitle").to_string(),
        body: t!("testPushNotificationBody").to_string(),
    };

    push_notification_server
        .send(&[token], &notification)
        .await?;

    Ok(())
}
