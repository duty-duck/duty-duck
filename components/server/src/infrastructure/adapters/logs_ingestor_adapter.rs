use opentelemetry::proto::{
    collector::logs::v1::{logs_service_client::LogsServiceClient, ExportLogsServiceRequest},
    logs::v1::ResourceLogs,
};
use tonic::{metadata::MetadataValue, transport::Channel, Request};

use crate::domain::{
    entities::authorization::{AuthContext, OriginalAuthenticationToken},
    ports::logs::LogsIngestor,
};

#[derive(Clone)]
pub struct LogsIngestorAdapter {
    client: LogsServiceClient<Channel>,
}

#[async_trait::async_trait]
impl LogsIngestor for LogsIngestorAdapter {
    #[tracing::instrument(skip(self, auth_context))]
    async fn ingest_logs(
        &self,
        auth_context: &AuthContext,
        logs: Vec<ResourceLogs>,
    ) -> anyhow::Result<()> {
        let mut request = Request::new(ExportLogsServiceRequest {
            resource_logs: logs,
        });

        match &auth_context.original_auth_token {
            Some(OriginalAuthenticationToken::APITokenPair { id, secret_key }) => {
                request
                    .metadata_mut()
                    .insert("x-api-token-id", MetadataValue::try_from(&id.to_string())?);
                request.metadata_mut().insert(
                    "x-api-token-secret-key",
                    MetadataValue::try_from(secret_key)?,
                );
            }
            Some(OriginalAuthenticationToken::BearerToken { bearer_token }) => {
                request.metadata_mut().insert(
                    "authentication",
                    MetadataValue::try_from(&format!("Bearer {bearer_token}"))?,
                );
            }
            None => (),
        }

        self.client.clone().export(request).await?;
        Ok(())
    }
}

impl LogsIngestorAdapter {
    pub async fn new(ingestors_urls: Vec<String>) -> anyhow::Result<Self> {
        tracing::info!(urls = ?ingestors_urls,  "Creating logs ingestor adapter");

        let mut endpoints = vec![];
        for url in ingestors_urls {
            let url = tonic::transport::Uri::try_from(url)?;
            endpoints.push(Channel::builder(url));
        }

        let client = LogsServiceClient::new(Channel::balance_list(endpoints.into_iter()));

        Ok(Self { client })
    }
}
