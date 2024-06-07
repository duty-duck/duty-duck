pub mod email_confirmation;

use ::entity::{tenant::TenantId, user_account};
use anyhow::{anyhow, Context};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use email_address::EmailAddress;
use email_confirmation::EmailConfirmationToken;
use sea_orm::*;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::{app_env::AppConfig, mailer::Mailer};

pub struct AuthService {
    app_config: Arc<AppConfig>,
    db: DatabaseConnection,
    mailer: Mailer,
    argon: Argon2<'static>,
}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("We could not find any account for these credentials. Please verify your e-mail address or sign up.")]
    UserNotFound,
    #[error("Your credentials are invalid.")]
    InvalidPassword,
    #[error("We need to verify your e-mail address before you can log in.")]
    UserNotConfirmed { user_id: Uuid },
    #[error(transparent)]
    TechnicalError(#[from] anyhow::Error),
}

impl AuthService {
    pub fn new(app_config: Arc<AppConfig>, db: DatabaseConnection, mailer: Mailer) -> Self {
        Self {
            db,
            mailer,
            app_config,
            argon: Argon2::default(),
        }
    }

    pub async fn get_user_by_id(
        &self,
        tenant: TenantId,
        id: Uuid,
    ) -> anyhow::Result<Option<user_account::Model>> {
        Ok(user_account::Entity::find_by_id((tenant, id))
            .one(&self.db)
            .await?)
    }

    pub async fn log_in(
        &self,
        tenant_id: TenantId,
        email: &str,
        password: &str,
    ) -> Result<user_account::Model, LoginError> {
        let user = user_account::Entity::find()
            .filter(user_account::Column::TenantId.eq(tenant_id))
            .filter(user_account::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(|e| LoginError::TechnicalError(anyhow!("SQL query failed: {e}")))?
            .ok_or(LoginError::UserNotFound)?;

        let password_is_valid = self.check_password(&user.password, password)?;
        if password_is_valid {
            let is_user_confirmed = user.email_confirmed_at.is_some();
            if is_user_confirmed {
                Ok(user)
            } else {
                Err(LoginError::UserNotConfirmed { user_id: user.id })
            }
        } else {
            Err(LoginError::InvalidPassword)
        }
    }

    fn check_password(&self, hashed_password: &str, password_input: &str) -> anyhow::Result<bool> {
        let password = PasswordHash::new(hashed_password)
            .map_err(|_| anyhow!("Failed to parse stored password"))?;
        Ok(self
            .argon
            .verify_password(password_input.as_bytes(), &password)
            .is_ok())
    }
}
