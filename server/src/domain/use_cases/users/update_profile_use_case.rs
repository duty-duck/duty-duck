use std::str::FromStr;

use anyhow::anyhow;
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use zxcvbn::{zxcvbn, Score};

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        user::{UpdateUserCommand, UpdateUserError, User},
    },
    ports::user_repository::UserRepository,
};

#[derive(Debug, Error)]
pub enum UpdateProfileError {
    #[error("e-mail is invalid")]
    InvalidEmail,
    #[error("password too weak")]
    PasswordTooWeak,
    #[error("Failed to update user profile: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateProfileCommand {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateProfileResponse {
    needs_session_invalidation: bool,
    new_user: User,
    new_user_permissions: Vec<Permission>,
}

pub async fn update_user_profile(
    auth_context: &AuthContext,
    repository: &impl UserRepository,
    command: UpdateProfileCommand,
) -> Result<UpdateProfileResponse, UpdateProfileError> {
    let needs_session_invalidation = command.password.is_some() || command.email.is_some();

    let first_name = command
        .first_name
        .as_deref()
        .or(auth_context.first_name.as_deref());
    let last_name = command
        .last_name
        .as_deref()
        .or(auth_context.last_name.as_deref());

    // Check the new e-mail is valid
    if let Some(email) = &command.email {
        if EmailAddress::from_str(email).is_err() {
            return Err(UpdateProfileError::InvalidEmail);
        }
    }

    // Check the new password is valid
    if let Some(new_password) = &command.password {
        let password_entropy = zxcvbn(
            new_password,
            &[
                first_name.unwrap_or_default(),
                last_name.unwrap_or_default(),
            ],
        );
        if password_entropy.score() < Score::Three {
            return Err(UpdateProfileError::PasswordTooWeak);
        };
    }

    let repo_command = UpdateUserCommand {
        first_name: command.first_name,
        last_name: command.last_name,
        email: command.email,
        password: command.password,
        phone_number: command.phone_number,
    };

    let new_user = match repository
        .update_user(auth_context.active_user_id, repo_command)
        .await
    {
        Ok(new_user) => new_user,
        Err(UpdateUserError::TechnicalFailure(e)) => {
            return Err(UpdateProfileError::TechnicalFailure(e))
        }
        Err(UpdateUserError::UserNotFound) => return Err(anyhow!("User not found").into()),
    };

    Ok(UpdateProfileResponse {
        needs_session_invalidation,
        new_user,
        new_user_permissions: Permission::iter_variants()
            .filter(|p| auth_context.can(*p))
            .collect(),
    })
}
