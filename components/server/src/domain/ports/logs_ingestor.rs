use opentelemetry_proto::tonic::logs::v1::ResourceLogs;

/// A port for the ingestor service in charge of ingesting logs (in the Open Telemetry format)
#[async_trait::async_trait]
pub trait LogsIngestor {
    async fn ingest_logs(&self, logs: Vec<ResourceLogs>) -> anyhow::Result<()>;
}
