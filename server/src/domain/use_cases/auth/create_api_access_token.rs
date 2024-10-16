use chrono::{DateTime, Days, Months, Utc};
use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::entities::authorization::{ApiAccessToken, AuthContext, Permission};

#[derive(Debug, Serialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiTokenRequest {
    pub label: String,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<Permission>,
}

#[derive(Debug, Serialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiTokenResponse {
    pub id: Uuid,
    pub secret_key: String,
}

#[derive(Debug, Error)]
pub enum CreateApiAccessTokenError {
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Token expiration date is in the past or too far in the future")]
    InvalidExpirationDate,
}

pub async fn create_api_access_token(
    auth_context: &AuthContext,
    request: CreateApiTokenRequest,
) -> Result<CreateApiTokenResponse, CreateApiAccessTokenError> {
    // Check if the user has the necessary permissions
    for scope in &request.scopes {
        if !auth_context.can(*scope) {
            return Err(CreateApiAccessTokenError::InsufficientPermissions);
        }
    }

    // Check if the expiration date is in the past. The expiration date must be at least 1 day in the future.
    if request.expires_at <= Utc::now() + Days::new(1) {
        return Err(CreateApiAccessTokenError::InvalidExpirationDate);
    }

    // Check if the expiration date is too far in the future. The expiration date must be within 18 months from now.
    if request.expires_at > Utc::now() + Months::new(18) {
        return Err(CreateApiAccessTokenError::InvalidExpirationDate);
    }

    let (secret_key_prefix, secret_key) = ApiAccessToken::generate_secret_key();

    // TODO: use a repository to store the token
    let id = todo!();
    let encoded_secret_key = ApiAccessToken::encode_secret_key(&secret_key_prefix, &secret_key);

    Ok(CreateApiTokenResponse {
        id,
        secret_key: encoded_secret_key,
    })
}
