mod email_confirmation;

use anyhow::{anyhow, Context};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use askama::Template;
use chrono::Utc;
use email_address::EmailAddress;
use entity::user_account;
use rusty_paseto::core::{Local, PasetoSymmetricKey, V4};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set, SqlErr};
use thiserror::Error;

use crate::mailer::Mailer;

pub struct AuthService {
    db: DatabaseConnection,
    paseto_key: PasetoSymmetricKey<V4, Local>,
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
struct SignupConfirmationEmail<'u> {
    user: &'u user_account::ActiveModel,
}

#[derive(Error, Debug)]
pub enum SignUpError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error(transparent)]
    TechnicalError(#[from] anyhow::Error),
}

pub type SignUpResult = Result<(), SignUpError>;

impl AuthService {
    pub fn new(
        db: DatabaseConnection,
        mailer: Mailer,
        paseto_key: PasetoSymmetricKey<V4, Local>,
    ) -> Self {
        Self {
            db,
            mailer,
            paseto_key,
            argon: Argon2::default(),
        }
    }

    pub async fn sign_up(&self, params: SignUpParams) -> SignUpResult {
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

        let user = user.save(&self.db).await.map_err(|e| match e.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => SignUpError::UserAlreadyExists,
            _ => SignUpError::TechnicalError(e.into()),
        })?;

        self.send_confirmation_email(&user)
            .await
            .with_context(|| "Failed to send confirmation e-mail")?;

        Ok(())
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
        user: &user_account::ActiveModel,
    ) -> anyhow::Result<()> {
        let body = SignupConfirmationEmail { user }.render()?;
        let message = Mailer::builder()
            .subject("Confirm your Duty Duck registration")
            .to(format!("{} <{}>", user.full_name.as_ref(), user.email.as_ref()).parse()?)
            .body(body)?;
        self.mailer.send(message).await
    }
}
