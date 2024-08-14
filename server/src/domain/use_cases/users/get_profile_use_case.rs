use thiserror::Error;

use crate::domain::{
    entities::{authorization::AuthContext, user::User},
    ports::user_repository::UserRepository,
};

#[derive(Debug, Error)]
pub enum GetProfileError {
    #[error("the user does not exist")]
    NotFound,
    #[error("Failed to update user profile: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn get_user_profile(
    auth_context: &AuthContext,
    repository: &impl UserRepository,
) -> Result<User, GetProfileError> {
    match repository.get_user(auth_context.active_user_id).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(GetProfileError::NotFound),
        Err(e) => Err(GetProfileError::TechnicalFailure(e)),
    }
}
