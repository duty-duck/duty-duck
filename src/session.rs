use std::str::FromStr;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::SET_COOKIE, request::Parts},
    response::{AppendHeaders, IntoResponseParts, Redirect},
};
use entity::user_account;
use headers::{Cookie, HeaderMapExt};
use rand::{rngs::OsRng, Rng};
use rusty_paseto::{
    core::{Local, V4},
    generic::{AudienceClaim, SubjectClaim, TokenIdentifierClaim},
    prelude::{PasetoBuilder, PasetoParser},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app_env::{AppConfig, AppEnv},
    crypto::SymetricEncryptionKey,
};

const SESSION_COOKIE_NAME: &str = "dutyducksession";

/// The [Session] struct contains the id of the currently logged-in user
/// It can be used directly as an extractor in Axum routes to access the current session
#[derive(Deserialize, Debug)]
pub struct Session {
    #[serde(rename = "sub")]
    pub user_id: Uuid,
    #[serde(rename = "jti")]
    pub csrf_token: CSRFToken,
}

impl Session {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            csrf_token: CSRFToken::new(),
        }
    }
}

/// Can be used in an Axum response to update the current session
pub struct SetSession<'config>(pub Session, pub &'config AppConfig);

impl<'c> IntoResponseParts for SetSession<'c> {
    type Error = String;

    fn into_response_parts(
        self,
        res: axum::response::ResponseParts,
    ) -> Result<axum::response::ResponseParts, Self::Error> {
        let session_token =
            SessionToken::encode(&self.1.paseto_key, self.0).map_err(|e| e.to_string())?;
        let header = (
            SET_COOKIE,
            format!(
                "{}={}; Secure; HttpOnly; Path=/;",
                SESSION_COOKIE_NAME, session_token.value
            ),
        );
        AppendHeaders::into_response_parts(AppendHeaders([header]), res).map_err(|e| e.to_string())
    }
}

/// Can be used in an Axum response to clear the current session
pub struct ClearSession;

impl IntoResponseParts for ClearSession {
    type Error = String;

    fn into_response_parts(
        self,
        res: axum::response::ResponseParts,
    ) -> Result<axum::response::ResponseParts, Self::Error> {
        let header = (
            SET_COOKIE,
            format!("{}=deleted; Secure; HttpOnly; Path=/;", SESSION_COOKIE_NAME,),
        );
        AppendHeaders::into_response_parts(AppendHeaders([header]), res).map_err(|e| e.to_string())
    }
}

#[async_trait]
impl FromRequestParts<AppEnv> for Session {
    type Rejection = Redirect;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppEnv,
    ) -> Result<Self, Self::Rejection> {
        let cookie = parts
            .headers
            .typed_get::<Cookie>()
            .and_then(|c| c.get(SESSION_COOKIE_NAME).map(|v| v.to_string()));

        match cookie {
            Some(value) if value != "deleted" => {
                SessionToken::decode(&state.config.paseto_key, SessionToken { value })
                    .ok()
                    .ok_or(Redirect::to("/auth/login"))
            }
            _ => Err(Redirect::to("/auth/login")),
        }
    }
}

/// An extractor that let's endpoints access the current user
pub struct CurrentUser(pub user_account::Model);

#[async_trait]
impl FromRequestParts<AppEnv> for CurrentUser {
    type Rejection = Redirect;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppEnv,
    ) -> Result<Self, Self::Rejection> {
        let Session { user_id, .. } = FromRequestParts::from_request_parts(parts, state).await?;
        state
            .auth_service
            .get_user_by_id(user_id)
            .await
            .ok()
            .flatten()
            .map(CurrentUser)
            .ok_or(Redirect::to("/auth/login"))
    }
}

/// A [SessionToken] is a PASETO token containing a serialized [Session] object
struct SessionToken {
    value: String,
}

impl SessionToken {
    fn encode(key: &SymetricEncryptionKey, session: Session) -> anyhow::Result<Self> {
        let sub = session.user_id.to_string();
        let jti = session.csrf_token.to_string();
        let value = PasetoBuilder::<V4, Local>::default()
            .set_claim(SubjectClaim::from(sub.as_str()))
            .set_claim(AudienceClaim::from("session"))
            .set_claim(TokenIdentifierClaim::from(jti.as_str()))
            .build(key)?;
        let value = urlencoding::encode(&value).into_owned();
        Ok(Self { value })
    }

    fn decode(key: &SymetricEncryptionKey, token: Self) -> anyhow::Result<Session> {
        let token = urlencoding::decode(&token.value)?;
        let value = PasetoParser::<V4, Local>::default()
            .check_claim(AudienceClaim::from("session"))
            .parse(&token, key)?;
        Ok(serde_json::from_value::<Session>(value)?)
    }
}

/// A 128-bit randomly-generated "synchrnizer token" stored in the session and included
/// in forms to prevent CSRF attacks
/// [OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html#synchronizer-token-pattern)
/// provides good referecne on CSRF attacks and the available prevention methods
#[derive(PartialEq, Eq, Deserialize, Debug)]
#[serde(transparent)]
pub struct CSRFToken(#[serde(with = "hex::serde")] [u8; 16]);

impl FromStr for CSRFToken {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut value = [0u8; 16];
        hex::decode_to_slice(s, &mut value)?;
        Ok(Self(value))
    }
}

impl std::fmt::Display for CSRFToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl CSRFToken {
    pub fn new() -> Self {
        Self(OsRng.gen::<u128>().to_le_bytes())
    }
}
