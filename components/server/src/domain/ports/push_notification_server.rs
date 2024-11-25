use crate::domain::entities::push_notification::{PushNotification, PushNotificationToken};

#[async_trait::async_trait]
pub trait PushNotificationServer: Clone + Send + Sync + 'static {
    async fn send(
        &self,
        devices_tokens: &[PushNotificationToken],
        notification: &PushNotification,
    ) -> anyhow::Result<()>;
}
