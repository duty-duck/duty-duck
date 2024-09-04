use std::{collections::HashMap, time::Instant};

use openidconnect::core::CoreIdToken;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub(super) struct AccessToken {
    pub(super) access_token: openidconnect::AccessToken,
    pub(super) expires_at: Option<Instant>,
    pub(super) refresh_token: Option<openidconnect::RefreshToken>,
    #[allow(unused)]
    pub(super) id_token: Option<CoreIdToken>,
}

impl AccessToken {
    pub(super) fn is_expired(&self) -> bool {
        self.expires_at.filter(|i| *i <= Instant::now()).is_some()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub domains: Vec<String>,
    pub url: Option<String>,
    pub attributes: AttributeMap,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserItem {
    pub id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub enabled: bool,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub attributes: AttributeMap,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub enabled: bool,
    pub groups: Vec<String>,
    pub attributes: AttributeMap,
    pub credentials: Vec<Credentials>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub enabled: Option<bool>,
    pub groups: Option<Vec<String>>,
    pub attributes: Option<AttributeMap>,
    pub credentials: Option<Vec<Credentials>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Credentials {
    #[serde(rename = "type")]
    pub credentials_type: CredentialsType,
    pub value: String,
    pub temporary: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum CredentialsType {
    Password,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WriteOrganizationRequest {
    pub name: String,
    pub display_name: String,
    pub url: Option<String>,
    pub domains: Vec<String>,
    pub attributes: AttributeMap,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrgnanizationRole {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(transparent)]
pub struct AttributeMap {
    pub map: HashMap<String, Vec<String>>,
}

impl AttributeMap {
    pub fn put(&mut self, key: &str, value: String) -> Option<String> {
        self.map
            .insert(key.to_string(), vec![value.to_string()])
            .and_then(|vec| vec.into_iter().next())
    }
    
    #[allow(unused)]
    pub fn put_json<T: Serialize>(&mut self, key: &str, value: &T) {
        if let Ok(v) = serde_json::to_string(value) {
            self.put(key, v);
        }
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.map
            .get(key)
            .and_then(|vec| vec.first())
            .map(|s| s.as_str())
    }
    pub fn get_json<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|s| serde_json::from_str(s).ok())
    }
}

#[macro_export]
macro_rules! attributes {
    ($($key:expr => $value:expr,)+) => { AttributeMap { map: maplit::hashmap!($($key => $value),+) } };
}

pub type Result<T> = core::result::Result<T, self::Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cannot obtain access token: {0}")]
    CannotObtainAccessToken(#[source] anyhow::Error),
    #[error("HTTP Error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Conflicting resource already exists")]
    Conflict,
    #[error("Resource not found")]
    NotFound,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}
