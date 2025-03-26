use anyhow::*;
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::Deserialize;
use uuid::Uuid;

use crate::{ClientError, DutyDuckApiClient, ResponseExtention};

#[derive(Clone)]
pub struct AuthSubclient {
    pub(crate) client: DutyDuckApiClient,
}

impl AuthSubclient {
    pub async fn get_current_user(&self) -> Result<GetProfileResponse, ClientError> {
        self.client
            .request(
                Method::GET,
                self.client
                    .base_url
                    .join("users/me")
                    .context("failed to build url")?,
            )?
            .send()
            .await?
            .json_or_err()
            .await
    }
}

#[derive(Deserialize, Debug)]
pub struct GetProfileResponse {
    pub user: User,
    pub active_organization: Organization,
    pub permissions: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub phone_number_verified: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
