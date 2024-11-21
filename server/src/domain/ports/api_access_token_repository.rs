use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::authorization::{ApiAccessToken, Permission};

#[derive(Clone)]
pub struct NewApiAccessToken {
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub label: String,
    pub secret_key: Vec<u8>,
    pub scopes: Vec<Permission>,
    pub expires_at: DateTime<Utc>,
}

#[async_trait]
pub trait ApiAccessTokenRepository: Send + Sync {
    async fn list_api_tokens(&self, organization_id: Uuid, user_id: Uuid) -> anyhow::Result<Vec<ApiAccessToken>>;

    async fn create_api_token(&self, token: NewApiAccessToken) -> anyhow::Result<Uuid>;

    async fn get_api_token(
        &self,
        token_id: Uuid,
    ) -> anyhow::Result<Option<ApiAccessToken>>;

    async fn delete_api_token(&self, user_id: Uuid, token_id: Uuid) -> anyhow::Result<bool>;
}
