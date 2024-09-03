use std::sync::Arc;

use anyhow::Context;
use gcp_auth::TokenProvider;
use serde::Serialize;

use crate::domain::{
    entities::push_notification::{PushNotification, PushNotificationToken},
    ports::push_notification_server::PushNotificationServer,
};

/// An adapter for the [PushNotificationServer] trait using Firebase Cloud Messaging
#[derive(Clone)]
pub struct PushNotificationServerAdapter {
    token_provider: Arc<dyn TokenProvider>,
    http_client: reqwest::Client,
}

impl PushNotificationServerAdapter {
    pub async fn new() -> anyhow::Result<Self> {
        let token_provider = gcp_auth::provider().await.with_context(|| "Failed to build GCP token provider. Maybe the GOOGLE_APPLICATION_CREDENTIALS env variable is not set.")?;
        Ok(Self {
            token_provider,
            http_client: reqwest::Client::new(),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn send(
        &self,
        PushNotificationToken(token): &PushNotificationToken,
        notification: &PushNotification,
    ) -> anyhow::Result<()> {
        let api_key = self
            .token_provider
            .token(&[
                "https://www.googleapis.com/auth/cloud-platform",
                "https://www.googleapis.com/auth/firebase.messaging",
            ])
            .await
            .with_context(|| "Failed to obtain Firebase auth token")?;

        let project_id = self.token_provider.project_id().await?;
        let request_body = MessageRequest {
            message: Message {
                token,
                notification: Notification {
                    title: &notification.title,
                    body: &notification.body,
                },
            },
        };

        let request = self
            .http_client
            .post(format!(
                "https://fcm.googleapis.com/v1/projects/{}/messages:send",
                project_id
            ))
            .bearer_auth(api_key.as_str())
            .json(&request_body);
        let response = request.send().await?;
        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();

            tracing::error!(
                status = status_code,
                body = text,
                "Failed HTTP Request to Firebase Cloud Messaging API"
            );

            anyhow::bail!("Failed HTTP Reqwest to Firebase Cloud messaging API, wrong HTTP Status");
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl PushNotificationServer for PushNotificationServerAdapter {
    async fn send(
        &self,
        devices_tokens: &[PushNotificationToken],
        notification: &PushNotification,
    ) -> anyhow::Result<()> {
        let futures = devices_tokens
            .iter()
            .map(|token| self.send(token, notification));
        futures_util::future::try_join_all(futures).await?;
        Ok(())
    }
}

#[derive(Serialize)]
struct MessageRequest<'a> {
    message: Message<'a>,
}

#[derive(Serialize)]
struct Message<'a> {
    notification: Notification<'a>,
    token: &'a str,
}

#[derive(Serialize)]
struct Notification<'a> {
    title: &'a str,
    body: &'a str,
}

#[cfg(test)]
#[tokio::test]
#[tracing_test::traced_test]
#[ignore]
pub async fn push_notification_adapter_test() {
    let adapter = PushNotificationServerAdapter::new().await.unwrap();
    // Change me to test other devices
    let token = PushNotificationToken("fEZxXj2wJH_GrN5zEDykOl:APA91bGezrWB-noWmLaiZ_MgDVrNOjeMe1-QDP4MxFj__ocLo7iixWW9eq4UTWWsJPWvisN9NRAQFmyBTBBfEqSDHuPprbI1eEcxKysWxLGpGP9XnW_3PJd3CSWiqRdY499-QZ1SZst4".to_string());
    let notification = PushNotification {
        title: "Test notification".to_string(),
        body: "Hello from Rust!".to_string(),
    };
    PushNotificationServer::send(&adapter, &[token], &notification).await.unwrap();
}
