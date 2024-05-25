pub mod email_confirmation;

use ::entity::user_account;
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

/// Params used to create a new user
pub struct SignUpParams {
    pub full_name: String,
    pub email: EmailAddress,
    pub password: String,
}

#[derive(Error, Debug)]
pub enum SignUpError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User already exists but their e-mail needs to be confirmed")]
    UnconfirmedUserAlreadyExists { user_id: Uuid },
    #[error(transparent)]
    TechnicalError(#[from] anyhow::Error),
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

pub type SignUpResult = Result<Uuid, SignUpError>;

impl AuthService {
    pub fn new(app_config: Arc<AppConfig>, db: DatabaseConnection, mailer: Mailer) -> Self {
        Self {
            db,
            mailer,
            app_config,
            argon: Argon2::default(),
        }
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> anyhow::Result<Option<user_account::Model>> {
        Ok(user_account::Entity::find_by_id(id).one(&self.db).await?)
    }

    pub async fn sign_up(&self, params: SignUpParams) -> SignUpResult {
        let existing_user = user_account::Entity::find()
            .filter(user_account::Column::Email.eq(params.email.as_str()))
            .one(&self.db)
            .await
            .map_err(|e| SignUpError::TechnicalError(e.into()))?;

        if let Some(existing_user) = existing_user {
            if existing_user.email_confirmed_at.is_none() {
                return Err(SignUpError::UnconfirmedUserAlreadyExists {
                    user_id: existing_user.id,
                });
            }
            return Err(SignUpError::UserAlreadyExists);
        }

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .argon
            .hash_password(params.password.as_bytes(), &salt)
            .map_err(|_| SignUpError::TechnicalError(anyhow!("Failed to hash password")))?;

        let now = Utc::now();
        let user = user_account::ActiveModel {
            full_name: Set(params.full_name),
            password: Set(password_hash.to_string()),
            email: Set(params.email.to_string()),
            updated_at: Set(now),
            created_at: Set(now),
            ..Default::default()
        };

        let user = user_account::Entity::insert(user)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| SignUpError::TechnicalError(e.into()))?;

        let confirmation_token =
            EmailConfirmationToken::build(&self.app_config.paseto_key, &user.id, &params.email)
                .with_context(|| "Failed to build e-mail confirmation token")?;

        self.send_confirmation_email(&user, &confirmation_token)
            .await
            .with_context(|| "Failed to send confirmation e-mail")?;

        Ok(user.id)
    }

    pub async fn log_in(
        &self,
        email: &str,
        password: &str,
    ) -> Result<user_account::Model, LoginError> {
        let user = user_account::Entity::find()
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
