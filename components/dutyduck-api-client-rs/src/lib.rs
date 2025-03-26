mod auth_subclient;
mod tasks_subclient;

use std::sync::Arc;

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
    base_url: Arc<reqwest::Url>,
    auth_token: Arc<Option<ApiToken>>,
}

#[derive(Default, Clone)]
struct ApiToken {
    id: String,
    secret_key: String,
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
            base_url: Arc::new(base_url.into_url().unwrap()),
            auth_token: Arc::new(None),
        }
    }

    pub fn from_reqwest_client(client: reqwest::Client, base_url: impl IntoUrl) -> Self {
        Self {
            client,
            base_url: Arc::new(base_url.into_url().unwrap()),
            auth_token: Arc::new(None),
        }
    }

    /// Sets the API token ID for authentication
    ///
    /// # Arguments
    /// * `token_id` - The API token ID string
    /// * `secret_key` - The API token secret key
    ///
    /// # Returns
    /// A new [`DutyDuckApiClient`] whose request are authenticated with this token.
    ///
    pub fn with_api_token(&self, token_id: String, secret_key: String) -> Self {
        Self {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            auth_token: Arc::new(Some(ApiToken {
                id: token_id,
                secret_key,
            })),
        }
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
        let auth_token = (*self.auth_token)
            .as_ref()
            .ok_or(ClientError::MissingApiToken)?;

        let builder = self
            .client
            .request(method, url)
            .header("X-Api-Token-Id", &auth_token.id)
            .header("X-Api-Token-Secret-Key", &auth_token.secret_key);

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
            let url = self.url().clone();
            let body = self
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".to_string());
            Err(ClientError::InvalidStatusCode(url, status, body))
        }
    }

    async fn json_or_err<T: DeserializeOwned>(self) -> ClientResult<T> {
        let status = self.status();
        if status.is_success() {
            Ok(self.json().await?)
        } else {
            let url = self.url().clone();
            let mut body = self.text().await.unwrap_or_default();
            body.truncate(1000);
            Err(ClientError::InvalidStatusCode(url, status, body))
        }
    }
}

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug, Error)]

pub enum ClientError {
    #[error("API token is not set")]
    MissingApiToken,
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("API endpoint '{0}' responded with an invalid status code: {1} and body: {2}")]
    InvalidStatusCode(reqwest::Url, reqwest::StatusCode, String),
}
