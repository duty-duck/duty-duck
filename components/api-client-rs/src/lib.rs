mod auth_subclient;
mod tasks_subclient;

use std::sync::{Arc, Mutex};

use anyhow::Context;
use async_trait::async_trait;
use auth_subclient::AuthSubclient;
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use tasks_subclient::TasksSubclient;

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
    ) -> anyhow::Result<reqwest::RequestBuilder> {
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
                auth_token
                    .id
                    .ok_or(anyhow::anyhow!("API token ID is not set"))?,
            )
            .header(
                "X-Api-Token-Secret-Key",
                auth_token
                    .secret_key
                    .ok_or(anyhow::anyhow!("API token secret key is not set"))?,
            );

        Ok(builder)
    }
}

#[async_trait]
pub trait ResponseExtention {
    async fn json_or_err<T: DeserializeOwned>(self) -> anyhow::Result<T>;
}

#[async_trait]
impl ResponseExtention for reqwest::Response {
    async fn json_or_err<T: DeserializeOwned>(self) -> anyhow::Result<T> {
        let status = self.status();
        if status.is_success() {
            self.json()
                .await
                .context("Failed to parse JSON response")
        } else {
            let body = self.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "API responded with an invalid status code: {}\nBody: {}",
                status,
                body
            ))
        }
    }
}
