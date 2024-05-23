use email_address::EmailAddress;
use rusty_paseto::{
    core::{Local, PasetoSymmetricKey, V4},
    generic::{AudienceClaim, CustomClaim, GenericParserError, PasetoClaimError, SubjectClaim},
    prelude::{PasetoBuilder, PasetoParser},
};
use serde::Deserialize;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::app_env::AppConfig;

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
    InvalidToken,
}

impl EmailConfirmationToken {
    pub fn url(app_config: &AppConfig, token: &Self) -> Url {
        app_config
            .public_url
            .join(&format!("/auth/confirm/{}", token.value))
            .unwrap()
    }

    pub fn build(
        key: &PasetoSymmetricKey<V4, Local>,
        user_id: &Uuid,
        email: &EmailAddress,
    ) -> anyhow::Result<Self> {
        let sub = user_id.to_string();
        let token = PasetoBuilder::<V4, Local>::default()
            .set_claim(SubjectClaim::from(sub.as_str()))
            .set_claim(CustomClaim::try_from(("email", email.as_str()))?)
            .set_claim(AudienceClaim::from("dutyduck-email-verification"))
            .build(key)?;
        let token = urlencoding::encode(&token).into_owned();

        Ok(Self { value: token })
    }

    pub fn decipher(
        key: &PasetoSymmetricKey<V4, Local>,
        token: Self,
    ) -> Result<DecipheredConfirmationToken, Error> {
        let token = urlencoding::decode(&token.value).map_err(|_| Error::InvalidToken)?;
        let value = PasetoParser::<V4, Local>::default()
            .check_claim(AudienceClaim::from("dutyduck-email-verification"))
            .parse(&token, key)
            .map_err(|e| match e {
                GenericParserError::ClaimError {
                    source: PasetoClaimError::Expired,
                } => Error::ExpiredToken,
                _ => Error::InvalidToken,
            })?;

        serde_json::from_value::<DecipheredConfirmationToken>(value)
            .map_err(|_| Error::InvalidToken)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use email_address::EmailAddress;
    use rusty_paseto::core::{Key, PasetoSymmetricKey};
    use uuid::Uuid;

    use super::EmailConfirmationToken;

    #[test]
    fn test_create_and_decipher_a_token() {
        let email = EmailAddress::from_str("foo@bar.com").unwrap();
        let key = PasetoSymmetricKey::from(Key::<32>::try_new_random().unwrap());
        let user_id = Uuid::new_v4();
        let confirmation_token = EmailConfirmationToken::build(&key, &user_id, &email).unwrap();
        let deciphered_token = EmailConfirmationToken::decipher(&key, confirmation_token).unwrap();
        assert_eq!(deciphered_token.user_id, user_id);
        assert_eq!(deciphered_token.email, email);
    }
}
