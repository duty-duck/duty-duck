mod keycloak_types;

use std::time::Duration;
use std::{sync::Arc, time::Instant};

pub use self::keycloak_types::*;
use anyhow::Context;
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::{ClientId, ClientSecret, IssuerUrl};
use openidconnect::{OAuth2TokenResponse, TokenResponse};
use reqwest::header::LOCATION;
use reqwest::{StatusCode, Url};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct KeycloakClient {
    keycloak_url: Url,
    keycloak_realm: String,
    realm_url: Url,
    realm_admin_url: Url,
    http_client: reqwest::Client,
    client_id: String,
    /// used to obtain service account tokens on behalf of the client
    client_secret: String,
    access_token: Arc<Mutex<Option<AccessToken>>>,
    oidc_client: CoreClient,
}

impl KeycloakClient {
    pub async fn new(
        keycloak_url: Url,
        keycloak_realm: &str,
        client_id: &str,
        client_secret: &str,
    ) -> anyhow::Result<Self> {
        let realm_url = keycloak_url.join(&format!("realms/{keycloak_realm}"))?;
        let realm_admin_url = keycloak_url.join(&format!("admin/realms/{keycloak_realm}"))?;
        // Use OpenID Connect Discovery to fetch the provider metadata.
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(realm_url.to_string())?,
            async_http_client,
        )
        .await?;

        let oidc_client: CoreClient = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(client_id.to_string()),
            Some(ClientSecret::new(client_secret.to_string())),
        );

        Ok(KeycloakClient {
            oidc_client,
            keycloak_url,
            keycloak_realm: keycloak_realm.to_string(),
            http_client: reqwest::Client::new(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            access_token: Arc::new(Mutex::default()),
            realm_url,
            realm_admin_url,
        })
    }

    pub async fn create_user(&self, user: &CreateUserRequest) -> Result<()> {
        let response = self
            .http_client
            .post(format!("{}/users", self.realm_admin_url))
            .json(user)
            .bearer_auth(self.get_current_access_token().await?.access_token.secret())
            .send()
            .await?;

        if response.status() == StatusCode::CONFLICT {
            Err(Error::Conflict)
        } else {
            response.error_for_status()?;
            Ok(())
        }
    }

    /**
     * Fetch all the organizations in the realm
     * # Parameters
     * - query: search by attributes using the format `k1:v1,k2:v2`
     */
    pub async fn get_organizations(
        &self,
        first: u32,
        max: u32,
        query: &str,
    ) -> Result<Vec<Organization>> {
        let orgs = self
            .http_client
            .get(format!("{}/orgs", self.realm_url))
            .query(&[
                ("first", first.to_string()),
                ("max", max.to_string()),
                ("q", query.to_string()),
            ])
            .bearer_auth(self.get_current_access_token().await?.access_token.secret())
            .send()
            .await?
            .json()
            .await?;
        Ok(orgs)
    }

    pub async fn create_organization(&self, request: &CreateOrganizationRequest) -> Result<Organization> {
        let response = self
            .http_client
            .post(format!("{}/orgs", self.realm_url))
            .json(request)
            .bearer_auth(self.get_current_access_token().await?.access_token.secret())
            .send()
            .await?;

        if response.status() == StatusCode::CONFLICT {
            Err(Error::Conflict)
        } else {
            let response = response.error_for_status()?;
            let location_header = response
                .headers()
                .get(LOCATION)
                .with_context(|| "Cannot get location header from response")?
                .to_str()
                .with_context(|| "Cannot read location header as str")?;

            let org = self
                .http_client
                .get(location_header)
                .bearer_auth(self.get_current_access_token().await?.access_token.secret())
                .send()
                .await?
                .json()
                .await?;

            Ok(org)
        }
    }

    pub async fn list_organization_members(
        &self,
        org_id: Uuid,
        first: u32,
        max: u32,
    ) -> Result<Vec<UserItem>> {
        let orgs = self
            .http_client
            .get(format!("{}/orgs/{}/members", self.realm_url, org_id))
            .query(&[("first", first.to_string()), ("max", max.to_string())])
            .bearer_auth(self.get_current_access_token().await?.access_token.secret())
            .send()
            .await?
            .json()
            .await?;
        Ok(orgs)
    }

    pub async fn add_an_organization_member(&self, org_id: Uuid, user_id: Uuid) -> Result<()> {
        self.http_client
            .put(format!(
                "{}/orgs/{}/members/{}",
                self.realm_url, org_id, user_id
            ))
            .bearer_auth(self.get_current_access_token().await?.access_token.secret())
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Obtain an access token for the Keycloak API, either by reading a valid token from memory, or by exchanging client credentials
    /// with Keycloak for a new token
    async fn get_current_access_token(&self) -> Result<AccessToken> {
        let mut current_access_token = self.access_token.lock().await;

        match &*current_access_token {
            None => {
                let new_token = self
                    .obtain_access_token()
                    .await
                    .map_err(Error::CannotObtainAccessToken)?;
                *current_access_token = Some(new_token.clone());
                Ok(new_token)
            }
            Some(token) if token.is_expired() => match self.refresh_access_token(token).await {
                Ok(new_token) => {
                    *current_access_token = Some(new_token.clone());
                    Ok(new_token)
                }
                Err(_) => match self.obtain_access_token().await {
                    Ok(new_token) => {
                        *current_access_token = Some(new_token.clone());
                        Ok(new_token)
                    }
                    Err(e) => {
                        *current_access_token = None;
                        Err(Error::CannotObtainAccessToken(e))
                    }
                },
            },
            Some(token) => Ok(token.clone()),
        }
    }

    async fn obtain_access_token(&self) -> anyhow::Result<AccessToken> {
        let res = self
            .oidc_client
            .exchange_client_credentials()
            .request_async(async_http_client)
            .await?;
        let access_token = AccessToken {
            access_token: res.access_token().clone(),
            refresh_token: res.refresh_token().cloned(),
            expires_at: res
                .expires_in()
                .map(|duration| Instant::now() + (duration - Duration::from_secs(2))),
            id_token: res.id_token().cloned(),
        };
        Ok(access_token)
    }

    async fn refresh_access_token(
        &self,
        access_token: &AccessToken,
    ) -> anyhow::Result<AccessToken> {
        let res = self
            .oidc_client
            .exchange_refresh_token(
                access_token
                    .refresh_token
                    .as_ref()
                    .with_context(|| "no refresh token available in access token")?,
            )
            .request_async(async_http_client)
            .await?;
        let access_token = AccessToken {
            access_token: res.access_token().clone(),
            refresh_token: res.refresh_token().cloned(),
            expires_at: res
                .expires_in()
                .map(|duration| Instant::now() + (duration - Duration::from_secs(2))),
            id_token: res.id_token().cloned(),
        };
        Ok(access_token)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;
    use nanoid::nanoid;

    use crate::domain::entities::organization::Address;

    use super::*;

    async fn build_client() -> anyhow::Result<KeycloakClient> {
        let keycloak_url = Url::parse("http://localhost:8080")?;
        let client_id = "dutyduck-server";
        let client_secret = "4ckxWkPRKGOrjvhtKtnbIX8T8awpLVMx";
        let keycloak_realm = "master";

        KeycloakClient::new(keycloak_url, keycloak_realm, client_id, client_secret).await
    }

    #[tokio::test]
    #[ignore]
    async fn test_obtain_token() -> anyhow::Result<()> {
        let client = build_client().await?;
        let access_token = client.obtain_access_token().await?;
        println!("{:#?}", access_token);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_refresh_token() -> anyhow::Result<()> {
        let client = build_client().await?;

        let access_token = client.obtain_access_token().await?;
        let access_token = client.refresh_access_token(&access_token).await?;
        println!("{:#?}", access_token);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_organizations() -> anyhow::Result<()> {
        let client = build_client().await?;
        let orgs = client.get_organizations(0, 10, "").await?;
        println!("{:#?}", orgs);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_organization() -> anyhow::Result<()> {
        let client = build_client().await?;
        let request = CreateOrganizationRequest {
            name: format!("test-organization-{}", nanoid!(10)),
            display_name: "Test organization".to_string(),
            url: None,
            domains: vec![],
            attributes: OrgAttributes {
                stripe_customer_id: Attribute::empty(),
                billing_address: Attribute::new(Address {
                    line_1: "Foo".to_string(),
                    line_2: "Bar".to_string(),
                    ..Default::default()
                }),
                created_at: Attribute::new(Utc::now()),
                updated_at: Attribute::new(Utc::now()),
                rest: HashMap::new(),
            },
        };
        client.create_organization(&request).await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_user() -> anyhow::Result<()> {
        let client = build_client().await?;
        let request = CreateUserRequest {
            email: Some("jane@noreply.com".to_string()),
            enabled: true,
            email_verified: true,
            first_name: Some("Jane".to_string()),
            last_name: Some("Doe".to_string()),
            attributes: UserAttributes::default(),
            groups: vec![],
            credentials: vec![Credentials {
                credentials_type: CredentialsType::Password,
                value: "1234".to_string(),
                temporary: false,
            }],
        };
        client.create_user(&request).await?;
        Ok(())
    }
}

pub type Result<T> = core::result::Result<T, self::Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cannot obtain access token")]
    CannotObtainAccessToken(#[source] anyhow::Error),
    #[error("HTTP Error")]
    HttpError(#[from] reqwest::Error),
    #[error("Conflicting resource already exists")]
    Conflict,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}
