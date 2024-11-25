use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    entities::authorization::AuthContext,
    ports::api_access_token_repository::ApiAccessTokenRepository,
};

#[derive(Debug, Error)]
pub enum DeleteApiAccessTokenError {
    #[error("Failed to persist API token: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
    #[error("API token not found")]
    ApiTokenNotFound,
}

pub async fn delete_api_access_token(
    auth_context: &AuthContext,
    repository: &impl ApiAccessTokenRepository,
    access_token_id: Uuid,
) -> Result<(), DeleteApiAccessTokenError> {
    match repository
        .delete_api_token(auth_context.active_user_id, access_token_id)
        .await?
    {
        true => Ok(()),
        false => Err(DeleteApiAccessTokenError::ApiTokenNotFound),
    }
}
