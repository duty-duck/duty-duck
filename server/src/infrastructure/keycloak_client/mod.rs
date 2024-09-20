mod keycloak_types;

#[cfg(test)]
mod tests;

use std::time::Duration;
use std::{sync::Arc, time::Instant};

pub use self::keycloak_types::*;
use anyhow::{anyhow, Context};
use backon::{ExponentialBuilder, Retryable};
use jsonwebtoken::jwk::JwkSet;
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::{ClientId, ClientSecret, IssuerUrl};
use openidconnect::{OAuth2TokenResponse, TokenResponse};
use reqwest::header::LOCATION;
use reqwest::{StatusCode, Url};
use tokio::sync::Mutex;
use tracing::debug;
use uuid::Uuid;

#[derive(Clone)]
struct CachedJwks {
    keys: JwkSet,
    last_retrieved_at: Instant,
}

pub struct KeycloakClient {
    pub realm: String,
    private_realm_url: Url,
    private_realm_admin_url: Url,
    http_client: reqwest::Client,
    client_id: String,
    /// used to obtain service account tokens on behalf of the client
    access_token: Arc<Mutex<Option<AccessToken>>>,
    oidc_client: CoreClient,
    // used to verify access tokens from keycloak
    cached_jwks: Arc<Mutex<Option<CachedJwks>>>,
}

impl KeycloakClient {
    /// Creates a new KeycloakClient instance.
    ///
    /// Initializes the client with the provided Keycloak configuration and pre-loads an access token.
    pub async fn new(
        public_keycloak_url: Url,
        private_keycloak_url: Url,
        keycloak_realm: &str,
        client_id: &str,
        client_secret: &str,
    ) -> anyhow::Result<Self> {
        let public_realm_url = public_keycloak_url.join(&format!("realms/{keycloak_realm}"))?;
        let private_realm_url = private_keycloak_url.join(&format!("realms/{keycloak_realm}"))?;
        let private_realm_admin_url =
            private_keycloak_url.join(&format!("admin/realms/{keycloak_realm}"))?;

        // Use OpenID Connect Discovery to fetch the provider metadata.
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(public_realm_url.to_string())?,
            async_http_client,
        )
        .await?;

        let oidc_client: CoreClient = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(client_id.to_string()),
            Some(ClientSecret::new(client_secret.to_string())),
        );

        let client = KeycloakClient {
            oidc_client,
            http_client: reqwest::Client::new(),
            client_id: client_id.to_string(),
            access_token: Arc::new(Mutex::default()),
            cached_jwks: Arc::new(Mutex::default()),
            private_realm_url,
            private_realm_admin_url,
            realm: keycloak_realm.to_string(),
        };

        // Pre-load access token for subsequent requests
        let _ = client.get_current_access_token().await?;

        Ok(client)
    }

    /// Retrieves the JSON Web Key Set (JWKS) for token verification.
    ///
    /// Returns a cached version if available and not expired, otherwise fetches a new set.
    #[tracing::instrument(skip(self))]
    pub async fn get_jwks(&self) -> anyhow::Result<JwkSet> {
        let mut jwk = self.cached_jwks.lock().await;
        match &*jwk {
            Some(set) if set.last_retrieved_at.elapsed() < Duration::from_secs(600) => {
                Ok(set.keys.clone())
            }
            _ => {
                let new_jwks = self.fetch_jwks().await?;
                *jwk = Some(new_jwks.clone());
                Ok(new_jwks.keys)
            }
        }
    }

    /// Creates a new user in Keycloak.
    ///
    /// Returns the created user or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn create_user(&self, user: &CreateUserRequest) -> Result<UserItem> {
        let auth_token = self.get_current_access_token().await?;
        let url = format!("{}/users", self.private_realm_admin_url);
        let response = (|| {
            self.http_client
                .post(&url)
                .json(user)
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        if response.status() == StatusCode::CONFLICT {
            Err(Error::Conflict)
        } else {
            if !response.status().is_success() {
                let status = response.status().as_u16();
                let response_body = response.text().await.unwrap_or_default();
                return Err(Error::TechnicalFailure(anyhow!(
                    "Failed HTTP request with URL '{url}', status '{status}' and response body: '{response_body}'",
                )));
            }
            let headers = response.headers().clone();
            let response_body = response.text().await.unwrap_or_default();
            println!("{response_body}");

            let location_header = headers
                .get(LOCATION)
                .with_context(|| {
                    format!(
                        "Cannot get location header from response. Url: '{url}', response headers: {:#?}",
                        headers
                    )
                })?
                .to_str()
                .with_context(|| "Cannot read location header as str")?;

            let user = self
                .http_client
                .get(location_header)
                .bearer_auth(self.get_current_access_token().await?.access_token.secret())
                .send()
                .await?
                .json()
                .await?;

            Ok(user)
        }
    }

    /// Retrieves a user by their UUID.
    ///
    /// Returns the user details or a NotFound error if the user doesn't exist.
    #[tracing::instrument(skip(self))]
    pub(super) async fn get_user_by_id(&self, id: Uuid) -> Result<UserItem> {
        let auth_token = self.get_current_access_token().await?;
        let response = (|| {
            self.http_client
                .get(format!("{}/users/{}", self.private_realm_admin_url, id))
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        if response.status() == StatusCode::NOT_FOUND {
            Err(Error::NotFound)
        } else {
            Ok(response.json().await?)
        }
    }

    /// Retrieves a user by their email address.
    ///
    /// Returns the user details or a NotFound error if the user doesn't exist.
    #[tracing::instrument(skip(self))]
    pub(super) async fn get_user_by_email(&self, email: &str) -> Result<UserItem> {
        let auth_token = self.get_current_access_token().await?;
        let response = (|| {
            self.http_client
                .get(format!("{}/users", self.private_realm_admin_url))
                .query(&[("email", email), ("exact", "true")])
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        if response.status() == StatusCode::OK {
            let users: Vec<UserItem> = response.json().await?;
            users.into_iter().next().ok_or(Error::NotFound)
        } else {
            Err(Error::TechnicalFailure(anyhow!(
                "Invalid response from Keycloak server: status = {:?} and body = {:?}",
                response.status(),
                response.text().await.unwrap_or_default()
            )))
        }
    }

    /// Updates an existing user's information.
    ///
    /// Returns the updated user details or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn update_user(&self, id: Uuid, user: &UpdateUserRequest) -> Result<UserItem> {
        let auth_token = self.get_current_access_token().await?;
        let response = (|| {
            self.http_client
                .put(format!("{}/users/{}", self.private_realm_admin_url, id))
                .json(user)
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        debug!(user_id = ?id, keycloak_response_status = response.status().as_u16(), "Updated user, got response from Keycloak server");

        if response.status() == StatusCode::NOT_FOUND {
            Err(Error::NotFound)
        } else if response.status().is_success() {
            self.get_user_by_id(id).await
        } else {
            Err(Error::TechnicalFailure(anyhow!(
                "Invalid repsonse from Keycloak server: status = {:?} and body = {:?}",
                response.status(),
                response.text().await
            )))
        }
    }

    /// Fetches organizations in the realm based on the provided criteria.
    ///
    /// # Parameters
    /// - `first`: The index of the first result to return.
    /// - `max`: The maximum number of results to return.
    /// - `query`: Search query in the format `k1:v1,k2:v2`.
    #[tracing::instrument(skip(self))]
    pub(super) async fn get_organizations(
        &self,
        first: u32,
        max: u32,
        query: &str,
    ) -> Result<Vec<Organization>> {
        let auth_token = self.get_current_access_token().await?;
        let res = (|| {
            self.http_client
                .get(format!("{}/orgs", self.private_realm_url))
                .query(&[
                    ("first", first.to_string()),
                    ("max", max.to_string()),
                    ("q", query.to_string()),
                ])
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;
        let orgs = res.json().await?;
        Ok(orgs)
    }

    /// Retrieves a specific organization by its UUID.
    ///
    /// Returns the organization details or a NotFound error if it doesn't exist.
    #[tracing::instrument(skip(self))]
    pub(super) async fn get_organization(&self, id: Uuid) -> Result<Organization> {
        let auth_token = self.get_current_access_token().await?;
        let url = format!("{}/orgs/{}", self.private_realm_url, id);

        let response = (|| {
            self.http_client
                .get(&url)
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        match response.status() {
            StatusCode::OK => {
                let org = response.json().await?;
                Ok(org)
            }
            StatusCode::NOT_FOUND => Err(Error::NotFound),
            _ => {
                let status = response.status().as_u16();
                let response_body = response.text().await.unwrap_or_default();
                Err(Error::TechnicalFailure(anyhow!(
                    "Failed HTTP request with URL '{url}', status '{status}' and response body: '{response_body}'"
                )))
            }
        }
    }

    /// Deletes an organization by its UUID.
    ///
    /// Returns Ok(()) on success or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn delete_organization(&self, id: Uuid) -> Result<()> {
        let auth_token = self.get_current_access_token().await?;
        let url = format!("{}/orgs/{}", self.private_realm_url, id);

        let response = (|| {
            self.http_client
                .delete(&url)
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::NOT_FOUND => Err(Error::NotFound),
            _ => {
                let status = response.status().as_u16();
                let response_body = response.text().await.unwrap_or_default();
                Err(Error::TechnicalFailure(anyhow!(
                    "Failed HTTP request with URL '{url}', status '{status}' and response body: '{response_body}'"
                )))
            }
        }
    }

    /// Creates a new organization.
    ///
    /// Returns the created organization or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn create_organization(
        &self,
        request: &WriteOrganizationRequest<'_>,
    ) -> Result<Organization> {
        let url = format!("{}/orgs", self.private_realm_url);
        let auth_token = self.get_current_access_token().await?;
        let response = (|| {
            self.http_client
                .post(&url)
                .json(request)
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        if response.status() == StatusCode::CONFLICT {
            Err(Error::Conflict)
        } else {
            if !response.status().is_success() {
                let status = response.status().as_u16();
                let response_body = response.text().await.unwrap_or_default();
                return Err(Error::TechnicalFailure(anyhow!(
                    "Failed HTTP request with URL '{url}', status '{status}' and response body: '{response_body}'",
                )));
            }
            let location_header = response
                .headers()
                .get(LOCATION)
                .with_context(|| {
                    format!(
                        "Cannot get location header from response. Url: '{url}', response headers: {:#?}",
                        response.headers()
                    )
                })?
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

    /// Updates an existing organization's information.
    ///
    /// Returns Ok(()) on success or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn update_organization(
        &self,
        org_id: Uuid,
        request: &WriteOrganizationRequest<'_>,
    ) -> Result<()> {
        let auth_token = self.get_current_access_token().await?;
        let response = (|| {
            self.http_client
                .put(format!("{}/orgs/{}", self.private_realm_url, org_id))
                .json(request)
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        if response.status() == StatusCode::NOT_FOUND {
            Err(Error::NotFound)
        } else {
            response.error_for_status()?;
            Ok(())
        }
    }

    /// Lists members of a specific organization.
    ///
    /// # Parameters
    /// - `org_id`: The UUID of the organization.
    /// - `first`: The index of the first result to return.
    /// - `max`: The maximum number of results to return.
    #[tracing::instrument(skip(self))]
    pub(super) async fn list_organization_members(
        &self,
        org_id: Uuid,
        first: u32,
        max: u32,
    ) -> Result<Vec<UserItem>> {
        let auth_token = self.get_current_access_token().await?;
        let res = (|| {
            self.http_client
                .get(format!(
                    "{}/orgs/{}/members",
                    self.private_realm_url, org_id
                ))
                .query(&[("first", first.to_string()), ("max", max.to_string())])
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        let orgs = res.json().await?;
        Ok(orgs)
    }

    /// Adds a user to an organization.
    ///
    /// Returns Ok(()) on success or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn add_an_organization_member(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        let auth_token = self.get_current_access_token().await?;
        (|| {
            self.http_client
                .put(format!(
                    "{}/orgs/{}/members/{}",
                    self.private_realm_url, org_id, user_id
                ))
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?
        .error_for_status()?;

        Ok(())
    }

    /// Removes a user from an organization.
    ///
    /// Returns Ok(()) on success or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn remove_an_organization_member(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        let auth_token = self.get_current_access_token().await?;
        let response = (|| {
            self.http_client
                .delete(format!(
                    "{}/orgs/{}/members/{}",
                    self.private_realm_url, org_id, user_id
                ))
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        response.error_for_status()?;
        Ok(())
    }

    /// Invites a user to join an organization.
    ///
    /// Returns Ok(()) on success or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn invite_user_to_organization(
        &self,
        org_id: Uuid,
        invite_request: &InviteUserRequest,
    ) -> Result<Invitation> {
        let auth_token = self.get_current_access_token().await?;
        let url = format!("{}/orgs/{}/invitations", self.private_realm_url, org_id);
        let response = (|| {
            self.http_client
                .post(url.clone())
                .bearer_auth(auth_token.access_token.secret())
                .json(invite_request)
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        match response.status() {
            reqwest::StatusCode::CREATED => (),
            reqwest::StatusCode::CONFLICT => return Err(Error::Conflict),
            reqwest::StatusCode::NOT_FOUND => return Err(Error::NotFound),
            status => {
                let error_body = response.text().await.unwrap_or_default();
                return Err(Error::TechnicalFailure(anyhow::anyhow!(
                    "Failed to invite user. Status: {}. Body: {}",
                    status,
                    error_body
                )));
            }
        };

        let location_header = response
            .headers()
            .get(LOCATION)
            .with_context(|| {
                format!(
                    "Cannot get location header from response. Url: '{url}', response headers: {:#?}",
                    response.headers()
                )
            })?
            .to_str()
            .with_context(|| "Cannot read location header as str")?;

        let invitation = self
            .http_client
            .get(location_header)
            .bearer_auth(self.get_current_access_token().await?.access_token.secret())
            .send()
            .await?
            .json()
            .await?;

        Ok(invitation)
    }

    /// Creates a new role for an organization.
    ///
    /// Returns Ok(()) on success or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn create_an_organization_role(&self, org_id: Uuid, role: &str) -> Result<()> {
        let auth_token = self.get_current_access_token().await?;
        (|| {
            self.http_client
                .post(format!("{}/orgs/{}/roles", self.private_realm_url, org_id))
                .bearer_auth(auth_token.access_token.secret())
                .json(&OrgnanizationRole {
                    name: role.to_string(),
                })
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?
        .error_for_status()?;

        Ok(())
    }

    /// Grants a role to a user within an organization.
    ///
    /// Returns Ok(()) on success or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn grant_an_organization_role(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<()> {
        let auth_token = self.get_current_access_token().await?;
        let url = format!(
            "{}/orgs/{}/roles/{}/users/{}",
            self.private_realm_url, org_id, role, user_id
        );
        let response = (|| {
            self.http_client
                .put(url.clone())
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::OK | StatusCode::CREATED => Ok(()),
            StatusCode::NOT_FOUND => Err(Error::NotFound),
            _ => {
                let status = response.status().as_u16();
                let response_body = response.text().await.unwrap_or_default();
                Err(Error::TechnicalFailure(anyhow!(
                    "Failed HTTP request with URL '{url}', status '{status}' and response body: '{response_body}'"
                )))
            }
        }
    }

    /// Lists the organization roles of a user within an organization.
    ///
    /// Returns a Result containing a Vec of roles on success, or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn list_organization_roles_for_user(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<OrgnanizationRole>> {
        let auth_token = self.get_current_access_token().await?;
        let url = format!(
            "{}/users/{}/orgs/{}/roles",
            self.private_realm_url, user_id, org_id
        );
        let response = (|| {
            self.http_client
                .get(url.clone())
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        match response.status() {
            StatusCode::OK => {
                let roles: Vec<OrgnanizationRole> = response.json().await?;
                Ok(roles)
            }
            StatusCode::NOT_FOUND => Err(Error::NotFound),
            _ => {
                let status = response.status().as_u16();
                let response_body = response.text().await.unwrap_or_default();
                Err(Error::TechnicalFailure(anyhow!(
                    "Failed HTTP request with URL '{url}', status '{status}' and response body: '{response_body}'"
                )))
            }
        }
    }

    /// Revokes a role from a user within an organization.
    ///
    /// Returns Ok(()) on success or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn revoke_an_organization_role(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<()> {
        let auth_token = self.get_current_access_token().await?;
        (|| {
            self.http_client
                .delete(format!(
                    "{}/orgs/{}/roles/{}/users/{}",
                    self.private_realm_url, org_id, role, user_id
                ))
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?
        .error_for_status()?;

        Ok(())
    }

    /// Retrieves a single invitation by its ID within an organization.
    ///
    /// Returns the invitation on success or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn get_invitation_by_id(
        &self,
        org_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<Invitation> {
        let auth_token = self.get_current_access_token().await?;
        let response = (|| {
            self.http_client
                .get(format!(
                    "{}/orgs/{}/invitations/{}",
                    self.private_realm_url, org_id, invitation_id
                ))
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        if response.status() == StatusCode::NOT_FOUND {
            Err(Error::NotFound)
        } else {
            let invitation = response.error_for_status()?.json().await?;
            Ok(invitation)
        }
    }

    /// Removes a pending invitation by its ID within an organization.
    ///
    /// Returns Ok(()) on success or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn remove_invitation_by_id(
        &self,
        org_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<()> {
        let auth_token = self.get_current_access_token().await?;
        let response = (|| {
            self.http_client
                .delete(format!(
                    "{}/orgs/{}/invitations/{}",
                    self.private_realm_url, org_id, invitation_id
                ))
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        if response.status() == StatusCode::NOT_FOUND {
            Err(Error::NotFound)
        } else {
            response.error_for_status()?;
            Ok(())
        }
    }

    /// Lists pending invitations for an organization.
    ///
    /// Returns a vector of Invitation on success or an error if the operation fails.
    #[tracing::instrument(skip(self))]
    pub(super) async fn list_pending_invitations(
        &self,
        org_id: Uuid,
        first: u32,
        max: u32,
    ) -> Result<Vec<Invitation>> {
        let auth_token = self.get_current_access_token().await?;
        let response = (|| {
            self.http_client
                .get(format!(
                    "{}/orgs/{}/invitations?first={}&max={}",
                    self.private_realm_url, org_id, first, max
                ))
                .bearer_auth(auth_token.access_token.secret())
                .send()
        })
        .retry(&Self::retry_strategy())
        .await?;

        if response.status() == StatusCode::NOT_FOUND {
            Err(Error::NotFound)
        } else {
            let invitations: Vec<Invitation> = response.json().await?;
            Ok(invitations)
        }
    }

    /// Obtain an access token for the Keycloak API, either by reading a valid token from memory, or by exchanging client credentials
    /// with Keycloak for a new token
    #[tracing::instrument(skip(self))]
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
        let action = || {
            self.oidc_client
                .exchange_client_credentials()
                .request_async(async_http_client)
        };
        let res = action.retry(&Self::retry_strategy()).await?;

        let access_token = AccessToken {
            access_token: res.access_token().clone(),
            refresh_token: res.refresh_token().cloned(),
            expires_at: res
                .expires_in()
                .map(|duration| Instant::now() + (duration - Duration::from_secs(2))),
            id_token: res.id_token().cloned(),
        };
        debug!(
            client_id = self.client_id,
            "Obtained a new keycloak access token"
        );
        Ok(access_token)
    }

    async fn refresh_access_token(
        &self,
        access_token: &AccessToken,
    ) -> anyhow::Result<AccessToken> {
        let refresh_token = access_token
            .refresh_token
            .as_ref()
            .with_context(|| "no refresh token available in access token")?;

        let action = || {
            self.oidc_client
                .exchange_refresh_token(refresh_token)
                .request_async(async_http_client)
        };

        let res = action.retry(&Self::retry_strategy()).await?;
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

    async fn fetch_jwks(&self) -> anyhow::Result<CachedJwks> {
        let action = || {
            self.http_client
                .get(format!(
                    "{}/protocol/openid-connect/certs",
                    self.private_realm_url
                ))
                .send()
        };

        let keys = action
            .retry(&Self::retry_strategy())
            .await?
            .json::<JwkSet>()
            .await?;
        Ok(CachedJwks {
            keys,
            last_retrieved_at: Instant::now(),
        })
    }

    fn retry_strategy() -> ExponentialBuilder {
        ExponentialBuilder::default()
            .with_max_times(5)
            .with_jitter()
    }
}
