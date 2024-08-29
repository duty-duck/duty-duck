use anyhow::*;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    entities::user_device::UserDevice,
    ports::user_devices_repository::{NewUserDevice, UserDevicesRepository},
};

#[derive(Clone)]
pub struct UserDevicesRepositoryAdapter {
    pub pool: PgPool,
}

#[async_trait::async_trait]
impl UserDevicesRepository for UserDevicesRepositoryAdapter {
    async fn register_device(&self, device: NewUserDevice) -> anyhow::Result<Uuid> {
        let id = sqlx::query!(
            "INSERT INTO user_devices (organization_id, user_id, label, device_type, push_notification_token) values ($1, $2, $3, $4, $5) RETURNING id",
            device.organization_id,
            device.user_id,
            device.label,
            device.device_type as i16,
            device.push_notification_token
        )
        .fetch_one(&self.pool).await?.id;
        Ok(id)
    }

    async fn remove_device(&self, organization_id: Uuid, device_id: Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM user_devices WHERE organization_id = $1 AND id = $2",
            organization_id,
            device_id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_user_devices(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<UserDevice>> {
        sqlx::query_as!(
            UserDevice,
            "SELECT * FROM user_devices WHERE organization_id = $1 AND user_id = $2",
            organization_id,
            user_id,
        )
        .fetch_all(&self.pool)
        .await
        .with_context(|| "Failed to list user devices from the database")
    }
}
