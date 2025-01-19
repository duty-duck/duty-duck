use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

use crate::domain::{
    entities::authorization::{ApiAccessToken, AuthContext},
    ports::api_access_token_repository::ApiAccessTokenRepository,
};

#[derive(Debug, Error)]
pub enum ListApiAccessTokensError {
    #[error("Failed to persist API token: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ListApiAccessTokensResponse {
    pub api_tokens: Vec<ApiAccessToken>,
}

pub async fn list_api_access_tokens(
    auth_context: &AuthContext,
    repository: &impl ApiAccessTokenRepository,
) -> Result<ListApiAccessTokensResponse, ListApiAccessTokensError> {
    let api_tokens = repository
        .list_api_tokens(
            auth_context.active_organization_id,
            auth_context.active_user_id,
        )
        .await?;
    Ok(ListApiAccessTokensResponse { api_tokens })
}
