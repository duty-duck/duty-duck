pub mod email_confirmation;

use ::entity::user_account;
use anyhow::{anyhow, Context};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use askama::Template;
use chrono::Utc;
use email_address::EmailAddress;
use email_confirmation::EmailConfirmationToken;
use sea_orm::*;
use std::sync::Arc;
use thiserror::Error;
use tracing::debug;
use url::Url;
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

#[derive(Template)]
#[template(path = "emails/signup-confirmation.html")]
struct SignupConfirmationEmail {
    email_confirmation_url: Url,
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

pub type SignUpResult = Result<Uuid, SignUpError>;

pub enum ConfirmEmailError {
    UserAlreadyConfirmed { user_id: Uuid },
    InvalidToken,
    TechnicalIssue,
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

    pub async fn resend_confirmation_email(&self, user_id: Uuid) -> anyhow::Result<()> {
        let existing_user = user_account::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .with_context(|| "Cannot find user")?;
        let email = existing_user.email.parse::<EmailAddress>()?;

        let confirmation_token =
            EmailConfirmationToken::build(&self.app_config.paseto_key, &existing_user.id, &email)
                .with_context(|| "Failed to build e-mail confirmation token")?;

        self.send_confirmation_email(&existing_user, &confirmation_token)
            .await
    }

    pub async fn confirm_email(
        &self,
        token: EmailConfirmationToken,
    ) -> Result<Uuid, ConfirmEmailError> {
        let deciphered_token = EmailConfirmationToken::decipher(&self.app_config.paseto_key, token)
            .map_err(|e| match e {
                email_confirmation::Error::InvalidToken { details } => {
                    debug!(details = details, "An invalid token was supplied");
                    ConfirmEmailError::InvalidToken
                }
                email_confirmation::Error::ExpiredToken => {
                    debug!("An expired token was supplied");
                    ConfirmEmailError::InvalidToken
                }
            })?;

        let user = user_account::Entity::find_by_id(deciphered_token.user_id)
            .one(&self.db)
            .await
            .ok()
            .flatten()
            .ok_or_else(|| {
                debug!("A valid confirmation token was suppliged but the user cannot be found in the database");
                ConfirmEmailError::InvalidToken
            })?;

        if user.email_confirmed_at.is_some() {
            return Err(ConfirmEmailError::UserAlreadyConfirmed { user_id: user.id });
        }

        let mut user = user_account::ActiveModel::from(user);

        user.email_confirmed_at = Set(Some(Utc::now()));
        let user = user
            .update(&self.db)
            .await
            .map_err(|_| ConfirmEmailError::TechnicalIssue)?;

        Ok(user.id)
    }

    fn check_password(&self, hashed_password: &str, password_input: &[u8]) -> anyhow::Result<bool> {
        let password = PasswordHash::new(hashed_password)
            .map_err(|_| anyhow!("Failed to parse stored password"))?;
        Ok(self
            .argon
            .verify_password(password_input, &password)
            .is_ok())
    }

    async fn send_confirmation_email(
        &self,
        user: &user_account::Model,
        confirmation_token: &EmailConfirmationToken,
    ) -> anyhow::Result<()> {
        let body = SignupConfirmationEmail {
            email_confirmation_url: EmailConfirmationToken::url(
                &self.app_config,
                confirmation_token,
            ),
        }
        .render()?;
        let message = Mailer::builder()
            .subject("Confirm your Duty Duck registration")
            .to(format!("{} <{}>", user.full_name, user.email).parse()?)
            .body(body)?;
        self.mailer.send(message).await
    }
}
