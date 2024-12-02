mod auth_subclient;
mod tasks_subclient;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use thiserror::Error;

pub use auth_subclient::*;
pub use tasks_subclient::*;

/// A client for interacting with the DutyDuck API
#[derive(Clone)]
pub struct DutyDuckApiClient {
    client: reqwest::Client,
    base_url: reqwest::Url,
    auth_token: Arc<Mutex<ApiToken>>,
}

#[derive(Default, Clone)]
struct ApiToken {
    id: Option<String>,
    secret_key: Option<String>,
}

impl DutyDuckApiClient {
    /// Creates a new instance of the DutyDuck API client with default configuration
    ///
    /// # Examples
    /// ```
    /// let client = DutyDuckApiClient::new();
    /// ```
    pub fn new(base_url: impl IntoUrl) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into_url().unwrap(),
            auth_token: Arc::new(Mutex::new(ApiToken::default())),
        }
    }

    /// Sets the API token ID for authentication
    ///
    /// # Arguments
    /// * `token_id` - The API token ID string
    ///
    /// # Returns
    /// * `Ok(())` if the token ID was set successfully
    /// * `Err` if there was an error acquiring the lock
    ///
    /// # Examples
    /// ```
    /// let client = DutyDuckApiClient::new();
    /// client.set_api_token_id("my-token-id".to_string())?;
    /// ```
    pub fn set_api_token_id(&self, token_id: String) -> anyhow::Result<()> {
        let mut auth_token = self
            .auth_token
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock auth token"))?;
        auth_token.id = Some(token_id);
        Ok(())
    }

    /// Sets the API token secret key for authentication
    ///
    /// # Arguments
    /// * `secret_key` - The API token secret key string
    ///
    /// # Returns
    /// * `Ok(())` if the secret key was set successfully
    /// * `Err` if there was an error acquiring the lock
    ///
    /// # Examples
    /// ```
    /// let client = DutyDuckApiClient::new();
    /// client.set_api_token_secret_key("my-secret-key".to_string())?;
    /// ```
    pub fn set_api_token_secret_key(&self, secret_key: String) -> anyhow::Result<()> {
        let mut auth_token = self
            .auth_token
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock auth token"))?;
        auth_token.secret_key = Some(secret_key);
        Ok(())
    }

    /// Returns an authentication subclient for handling auth-related API operations
    ///
    /// # Returns
    /// An `AuthSubclient` instance bound to this API client
    ///
    /// # Examples
    /// ```
    /// let client = DutyDuckApiClient::new();
    /// let auth_client = client.auth();
    /// ```
    pub fn auth(&self) -> AuthSubclient {
        AuthSubclient {
            client: self.clone(),
        }
    }

    /// Returns a tasks subclient for handling task-related API operations
    ///
    /// # Returns
    /// A `TasksSubclient` instance bound to this API client
    ///
    /// # Examples
    /// ```
    /// let client = DutyDuckApiClient::new();
    /// let tasks_client = client.tasks();
    /// ```
    pub fn tasks(&self) -> TasksSubclient {
        TasksSubclient {
            client: self.clone(),
        }
    }

    pub(crate) fn request(
        &self,
        method: reqwest::Method,
        url: impl IntoUrl,
    ) -> ClientResult<reqwest::RequestBuilder> {
        let auth_token = {
            let lock = self
                .auth_token
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock auth token"))?;
            lock.clone()
        };

        let builder = self
            .client
            .request(method, url)
            .header(
                "X-Api-Token-Id",
                auth_token.id.ok_or(ClientError::MissingApiTokenId)?,
            )
            .header(
                "X-Api-Token-Secret-Key",
                auth_token
                    .secret_key
                    .ok_or(ClientError::MissingApiTokenSecretKey)?,
            );

        Ok(builder)
    }
}

#[async_trait]
pub trait ResponseExtention {
    async fn json_or_err<T: DeserializeOwned>(self) -> ClientResult<T>;
    async fn ok_or_err(self) -> ClientResult<()>;
}

#[async_trait]
impl ResponseExtention for reqwest::Response {
    async fn ok_or_err(self) -> ClientResult<()> {
        let status = self.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = self.text().await.unwrap_or_else(|_| "<no body>".to_string());
            Err(ClientError::InvalidStatusCode(status, body))
        }
    }

    async fn json_or_err<T: DeserializeOwned>(self) -> ClientResult<T> {
        let status = self.status();
        if status.is_success() {
            Ok(self.json().await?)
        } else {
            let body = self.text().await.unwrap_or_default();
            Err(ClientError::InvalidStatusCode(status, body))
        }
    }
}

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug, Error)]

pub enum ClientError {
    #[error("API token ID is not set")]
    MissingApiTokenId,
    #[error("API token secret key is not set")]
    MissingApiTokenSecretKey,
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("API responded with an invalid status code: {0} and body: {1}")]
    InvalidStatusCode(reqwest::StatusCode, String),
}
