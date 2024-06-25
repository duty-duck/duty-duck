use std::time::Duration;
use std::{sync::Arc, time::Instant};

use anyhow::Context;
use openidconnect::core::{CoreClient, CoreIdToken, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::{ClientId, ClientSecret, IssuerUrl};
use openidconnect::{OAuth2TokenResponse, TokenResponse};
use reqwest::{Method, Url};
use tokio::sync::Mutex;

pub struct KeycloakClient {
    keycloak_url: Url,
    keycloak_realm: String,
    http_client: reqwest::Client,
    client_id: String,
    /// used to obtain service account tokens on behalf of the client
    client_secret: String,
    access_token: Arc<Mutex<Option<AccessToken>>>,
    oidc_client: CoreClient,
}

#[derive(Clone, Debug)]
struct AccessToken {
    access_token: openidconnect::AccessToken,
    expires_at: Option<Instant>,
    refresh_token: Option<openidconnect::RefreshToken>,
    id_token: Option<CoreIdToken>,
}

impl AccessToken {
    fn is_expired(&self) -> bool {
        self.expires_at.filter(|i| *i <= Instant::now()).is_some()
    }
}

impl KeycloakClient {
    pub async fn new(
        keycloak_url: Url,
        keycloak_realm: &str,
        client_id: &str,
        client_secret: &str,
    ) -> anyhow::Result<Self> {
        // Use OpenID Connect Discovery to fetch the provider metadata.
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(
                keycloak_url
                    .join("realms")?
                    .join(keycloak_realm)?
                    .join(".well-known")?
                    .join("openid-configuration")?
                    .to_string(),
            )?,
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
        })
    }

    /// Obtain an access token for the Keycloak API, either by reading a valid token from memory, or by exchanging client credentials
    /// with Keycloak for a new token
    async fn get_current_access_token(&self) -> anyhow::Result<AccessToken> {
        let mut current_access_token = self.access_token.lock().await;

        match &*current_access_token {
            None => {
                let new_token = self.obtain_access_token().await?;
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
                        Err(e)
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
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_obtain_token() -> anyhow::Result<()> {
        let keycloak_url = Url::parse("http://localhost:8080")?;
        let client_id = "dutyduck-server";
        let client_secret = "4ckxWkPRKGOrjvhtKtnbIX8T8awpLVMx";
        let keycloak_realm = "master";

        let client = KeycloakClient::new(keycloak_url, keycloak_realm, client_id, client_secret).await?;
        let access_token = client.obtain_access_token().await?;
        println!("{:#?}", access_token);

        Ok(())

    }
}