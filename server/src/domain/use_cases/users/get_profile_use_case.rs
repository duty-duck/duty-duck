use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        user::User,
    },
    ports::user_repository::UserRepository,
};

#[derive(Debug, Error)]
pub enum GetProfileError {
    #[error("the user does not exist")]
    NotFound,
    #[error("Failed to update user profile: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct GetProfileResponse {
    user: User,
    permissions: Vec<Permission>,
}

pub async fn get_user_profile(
    auth_context: &AuthContext,
    repository: &impl UserRepository,
) -> Result<GetProfileResponse, GetProfileError> {
    let user = match repository.get_user(auth_context.active_user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(GetProfileError::NotFound),
        Err(e) => return Err(GetProfileError::TechnicalFailure(e)),
    };

    let response = GetProfileResponse {
        permissions: Permission::iter_variants()
            .filter(|p| auth_context.can(*p))
            .collect(),
        user,
    };

    Ok(response)
}
