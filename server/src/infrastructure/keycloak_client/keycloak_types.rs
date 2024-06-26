use std::{collections::HashMap, time::Instant};

use openidconnect::core::CoreIdToken;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationListItem {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub domains: Vec<String>,
    pub url: String,
    pub attributes: HashMap<String, Value>,
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
    pub attributes: HashMap<String, Value>,
}
