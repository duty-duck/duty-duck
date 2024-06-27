use std::{collections::HashMap, time::Instant};

use chrono::{DateTime, Utc};
use openidconnect::core::CoreIdToken;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use serde_with::{json::JsonString, serde_as};
use uuid::Uuid;

use crate::domain::entities::organization::Address;

#[derive(Clone, Debug)]
pub(super) struct AccessToken {
    pub(super) access_token: openidconnect::AccessToken,
    pub(super) expires_at: Option<Instant>,
    pub(super) refresh_token: Option<openidconnect::RefreshToken>,
    pub(super) id_token: Option<CoreIdToken>,
}

impl AccessToken {
    pub(super) fn is_expired(&self) -> bool {
        self.expires_at.filter(|i| *i <= Instant::now()).is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserAttributes {
    #[serde(flatten)]
    pub rest: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OrgAttributes {
    // keycloak requires that all custom attributes are sent as arrays of strings,
    // which is why we can't use Option<String> here, we have to wrap every attribute values inside vectors or arrays
    // so they are properly serialised
    #[serde(default)]
    pub stripe_customer_id: Attribute<String>,
    #[serde(default)]
    pub billing_address: Attribute<Address>,
    pub created_at: Attribute<DateTime<Utc>>,
    pub updated_at: Attribute<DateTime<Utc>>,
    #[serde(flatten)]
    pub rest: HashMap<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub domains: Vec<String>,
    pub url: Option<String>,
    pub attributes: OrgAttributes,
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
    pub groups: Vec<String>,
    pub attributes: UserAttributes,
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
    pub attributes: UserAttributes,
    pub credentials: Vec<Credentials>,
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
pub struct CreateOrganizationRequest {
    pub name: String,
    pub display_name: String,
    pub url: Option<String>,
    pub domains: Vec<String>,
    pub attributes: OrgAttributes,
}

#[serde_as]
#[serde(transparent)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Attribute<T: Serialize + DeserializeOwned> {
    #[serde_as(as = "Vec<JsonString>")]
    value: Vec<T>,
}

impl<T: Serialize + DeserializeOwned> Default for Attribute<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: Serialize + DeserializeOwned> Attribute<T> {
    pub fn new(value: T) -> Self {
        Self { value: vec![value] }
    }

    pub fn empty() -> Self {
        Self { value: vec![] }
    }

    pub fn get(&self) -> &T {
        &self.value[0]
    }

    pub fn unwrap(self) -> T {
        self.value.into_iter().next().unwrap()
    }
}

impl<T: Serialize + DeserializeOwned> From<Option<T>> for Attribute<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            None => Self::empty(),
            Some(v) => Self::new(v),
        }
    }
}
