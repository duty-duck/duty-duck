use ::entity::user_account;
use anyhow::Context;
use askama::Template;
use chrono::Utc;
use email_address::EmailAddress;
use rusty_paseto::{
    core::{Local, V4},
    generic::{AudienceClaim, CustomClaim, GenericParserError, PasetoClaimError, SubjectClaim},
    prelude::{PasetoBuilder, PasetoParser},
};
use sea_orm::*;
use serde::Deserialize;
use thiserror::Error;
use tracing::*;
use url::Url;
use uuid::Uuid;

use crate::{app_env::AppConfig, crypto::SymetricEncryptionKey, mailer::Mailer};

use super::AuthService;

#[derive(Template)]
#[template(path = "emails/signup-confirmation.html")]
struct SignupConfirmationEmail {
    email_confirmation_url: Url,
}

pub enum ConfirmEmailError {
    UserAlreadyConfirmed { user_id: Uuid },
    InvalidToken,
    TechnicalIssue,
}

impl AuthService {
    pub async fn resend_confirmation_email(&self, user_id: Uuid) -> anyhow::Result<()> {
        let existing_user = user_account::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .with_context(|| "Cannot find user")?;

        // User is already validated, no need to send an email
        // This would only happen if a user
        //   1. Loads the "re-send confirmatio e-mail" button in their browser
        //   2. Do click on the button
        //   3. Validate their account using some prior confirmation e-mail
        //   4. Attemps to click on the button once their account is already confirmed
        // We could provide a more precise feedback for this scenario, but for now, treating it as a success will do fine
        if existing_user.email_confirmed_at.is_some() {
            return Ok(());
        }

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
                self::Error::InvalidToken { details } => {
                    debug!(details = details, "An invalid token was supplied");
                    ConfirmEmailError::InvalidToken
                }
                self::Error::ExpiredToken => {
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

    pub(super) async fn send_confirmation_email(
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

pub struct EmailConfirmationToken {
    pub value: String,
}

#[derive(Deserialize)]
pub struct DecipheredConfirmationToken {
    pub email: EmailAddress,
    #[serde(rename = "sub")]
    pub user_id: Uuid,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("the token has expired, a new token must be generated")]
    ExpiredToken,
    #[error("the token is invalid")]
    InvalidToken { details: &'static str },
}

impl EmailConfirmationToken {
    pub fn url(app_config: &AppConfig, token: &Self) -> Url {
        app_config
            .public_url
            .join(&format!("/auth/confirm/{}", token.value))
            .unwrap()
    }

    pub fn build(
        key: &SymetricEncryptionKey,
        user_id: &Uuid,
        email: &EmailAddress,
    ) -> anyhow::Result<Self> {
        let sub = user_id.to_string();
        let token = PasetoBuilder::<V4, Local>::default()
            .set_claim(SubjectClaim::from(sub.as_str()))
            .set_claim(CustomClaim::try_from(("email", email.as_str()))?)
            .set_claim(AudienceClaim::from("email-verification"))
            .build(key)?;
        let token = urlencoding::encode(&token).into_owned();

        Ok(Self { value: token })
    }

    pub fn decipher(
        key: &SymetricEncryptionKey,
        token: Self,
    ) -> Result<DecipheredConfirmationToken, Error> {
        let token = urlencoding::decode(&token.value).map_err(|_| Error::InvalidToken {
            details: "failed to url decode",
        })?;
        let value = PasetoParser::<V4, Local>::default()
            .check_claim(AudienceClaim::from("email-verification"))
            .parse(&token, key)
            .map_err(|e| match e {
                GenericParserError::ClaimError {
                    source: PasetoClaimError::Expired,
                } => Error::ExpiredToken,
                _ => Error::InvalidToken {
                    details: "failed to parse PASETO token",
                },
            })?;

        serde_json::from_value::<DecipheredConfirmationToken>(value).map_err(|_| {
            Error::InvalidToken {
                details: "failed to deserialized the token's payload",
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use email_address::EmailAddress;
    use uuid::Uuid;

    use crate::crypto::SymetricEncryptionKey;

    use super::EmailConfirmationToken;

    #[test]
    fn test_create_and_decipher_a_token() {
        let email = EmailAddress::from_str("foo@bar.com").unwrap();
        let key = SymetricEncryptionKey::new_random();
        let user_id = Uuid::new_v4();
        let confirmation_token = EmailConfirmationToken::build(&key, &user_id, &email).unwrap();
        let deciphered_token = EmailConfirmationToken::decipher(&key, confirmation_token).unwrap();
        assert_eq!(deciphered_token.user_id, user_id);
        assert_eq!(deciphered_token.email, email);
    }
}
