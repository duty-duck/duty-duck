use std::{sync::Arc, time::Duration};

use anyhow::Context;
use dutyduck_api_client_rs::{ClientError, DutyDuckApiClient};

use moka::future::{Cache, CacheBuilder};
use reqwest::{IntoUrl, StatusCode};
use thiserror::Error;
use tonic::{Request, Status};
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthenticatedToken {
    pub org_id: Uuid,
    #[allow(unused)]
    pub org_name: String,
}

#[derive(Clone)]
pub struct Authenticator {
    api_client: DutyDuckApiClient,
    authenticated_token_cache: Arc<Cache<(String, String), AuthenticatedToken>>,
}

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("Could not authenticate your request. Your authentication token is invalid.")]
    FailedAuthentication,
    #[error("An internal failure occured while processing the request: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

impl From<AuthenticationError> for tonic::Status {
    fn from(value: AuthenticationError) -> Self {
        match value {
            AuthenticationError::FailedAuthentication => {
                Status::permission_denied(value.to_string())
            }
            AuthenticationError::TechnicalFailure(error) => {
                error!(?error, "Technical failure while authenticating a request");
                Status::internal("An internal error occured, please try again")
            }
        }
    }
}

impl Authenticator {
    pub fn new(server_url: impl IntoUrl) -> anyhow::Result<Self> {
        let server_url = server_url.into_url()?;

        // keep authenticated tokens in the cache for at most 5 minutes
        let cache = CacheBuilder::new(1000)
            .time_to_live(Duration::from_secs(600))
            .build();

        Ok(Self {
            api_client: DutyDuckApiClient::new(server_url),
            authenticated_token_cache: Arc::new(cache),
        })
    }

    /// Authenticates a given token id/secret key pair by calling the main server
    /// Authenticated tokens are put in an in-memory cache for 5 minutes to speed up further authentications
    pub async fn authenticate_token(
        &self,
        token_id: &str,
        token_secret_key: &str,
    ) -> Result<AuthenticatedToken, AuthenticationError> {
        let token_id = token_id.to_string();
        let token_secret_key = token_secret_key.to_string();
        let token_pair = (token_id, token_secret_key);

        if let Some(token) = self.authenticated_token_cache.get(&token_pair) {
            return Ok(token);
        }

        let client = self
            .api_client
            .with_api_token(token_pair.0.clone(), token_pair.1.clone());

        match client.auth().get_current_user().await {
            Ok(user) => {
                let authenticated_token = AuthenticatedToken {
                    org_id: user.active_organization.id,
                    org_name: user.active_organization.name,
                };

                self.authenticated_token_cache
                    .insert(token_pair, authenticated_token.clone())
                    .await;

                Ok(authenticated_token)
            }
            Err(ClientError::InvalidStatusCode(
                StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED,
                _,
            )) => Err(AuthenticationError::FailedAuthentication),
            Err(e) => Err(AuthenticationError::TechnicalFailure(e.into())),
        }
    }

    pub async fn authenticate_tonic_request<T>(
        &self,
        request: &Request<T>,
    ) -> Result<AuthenticatedToken, tonic::Status> {
        let metadata = request.metadata();
        let token_id = metadata
            .get("X-Api-Token-Id")
            .ok_or(AuthenticationError::FailedAuthentication)?
            .to_str()
            .context("Failed to convert token id to str")
            .map_err(AuthenticationError::TechnicalFailure)?;
        let secret_key = metadata
            .get("X-Api-Token-Secret-Key")
            .ok_or(AuthenticationError::FailedAuthentication)?
            .to_str()
            .context("Failed to convert secrey key to str")
            .map_err(AuthenticationError::TechnicalFailure)?;

        Ok(self.authenticate_token(token_id, secret_key).await?)
    }
}
