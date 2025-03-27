use uuid::Uuid;

use crate::domain::entities::user_device::{UserDevice, UserDeviceType};

#[async_trait::async_trait]
pub trait UserDevicesRepository: Clone + Send + Sync + 'static {
    async fn register_device(&self, device: NewUserDevice) -> anyhow::Result<Uuid>;

    /// Removes a user device, returns whether a device actually existed and was deleted
    async fn remove_device(&self, organization_id: Uuid, device_id: Uuid) -> anyhow::Result<bool>;

    async fn get_user_device(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
        device_id: Uuid,
    ) -> anyhow::Result<Option<UserDevice>> {
        let devices = self.list_user_devices(organization_id, user_id).await?;
        Ok(devices.into_iter().find(|d| d.id == device_id))
    }

    async fn list_user_devices(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<UserDevice>>;

    async fn list_organization_devices(
        &self,
        organization_id: Uuid,
    ) -> anyhow::Result<Vec<UserDevice>>;
}

#[derive(veil::Redact)]
pub struct NewUserDevice {
    pub organization_id: Uuid,
    pub user_id: Uuid,
    #[redact(partial)]
    pub label: String,
    #[redact]
    pub push_notification_token: Option<String>,
    pub device_type: UserDeviceType,
}
