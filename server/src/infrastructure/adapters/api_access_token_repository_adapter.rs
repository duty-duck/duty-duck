use async_trait::async_trait;
use sqlx::*;
use uuid::Uuid;

use crate::domain::{
    entities::authorization::{ApiAccessToken, Permission},
    ports::api_access_token_repository::{ApiAccessTokenRepository, NewApiAccessToken},
};

#[derive(Clone)]
pub struct ApiAccessTokenRepositoryAdapter {
    pub pool: PgPool,
}

#[async_trait]
impl ApiAccessTokenRepository for ApiAccessTokenRepositoryAdapter {
    async fn list_api_tokens(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<ApiAccessToken>> {
        let records = sqlx::query!(
            r#"
            SELECT * FROM api_access_tokens WHERE organization_id = $1 AND user_id = $2
            "#,
            organization_id,
            user_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|r| ApiAccessToken {
                id: r.id,
                organization_id: r.organization_id,
                user_id: r.user_id,
                label: r.label,
                // Erase the secret key
                secret_key: vec![],
                scopes: r
                    .scopes
                    .into_iter()
                    .map(Permission::from)
                    .collect(),
                expires_at: r.expires_at,
                created_at: r.created_at,
            })
            .collect())
    }

    async fn create_api_token(&self, token: NewApiAccessToken) -> anyhow::Result<Uuid> {
        let scopes: Vec<i16> = token.scopes.iter().map(|p| *p as i16).collect();

        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO api_access_tokens (
                organization_id, user_id, label, secret_key, expires_at, scopes
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            token.organization_id,
            token.user_id,
            token.label,
            token.secret_key,
            token.expires_at,
            &scopes as &[i16],
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    async fn get_api_token(
        &self,
        token_id: Uuid,
    ) -> anyhow::Result<Option<ApiAccessToken>> {
        let record = sqlx::query!(
            r#"
            SELECT 
                id, organization_id, user_id, label, secret_key, 
                created_at, expires_at, scopes as "scopes!"
            FROM api_access_tokens
            WHERE id = $1
            "#,
            token_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.map(|r| ApiAccessToken {
            id: r.id,
            organization_id: r.organization_id,
            user_id: r.user_id,
            label: r.label,
            secret_key: r.secret_key,
            scopes: r
                .scopes
                .into_iter()
                .map(Permission::from)
                .collect(),
            expires_at: r.expires_at,
            created_at: r.created_at,
        }))
    }

    async fn delete_api_token(&self, user_id: Uuid, token_id: Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM api_access_tokens
            WHERE id = $1 AND user_id = $2
            "#,
            token_id,
            user_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
