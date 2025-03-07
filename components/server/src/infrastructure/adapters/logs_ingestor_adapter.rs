use opentelemetry_proto::tonic::{
    collector::logs::v1::{logs_service_client::LogsServiceClient, ExportLogsServiceRequest},
    logs::v1::ResourceLogs,
};
use tonic::transport::Channel;

use crate::domain::ports::logs_ingestor::LogsIngestor;

#[derive(Clone)]
pub struct LogsIngestorAdapter {
    client: LogsServiceClient<Channel>,
}

#[async_trait::async_trait]
impl LogsIngestor for LogsIngestorAdapter {
    async fn ingest_logs(&self, logs: Vec<ResourceLogs>) -> anyhow::Result<()> {
        let request = ExportLogsServiceRequest {
            resource_logs: logs,
        };

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
