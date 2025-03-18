use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use anyhow::Context;
use auth::Authenticator;
use config::IngestorConfig;
use quickwit::QuickwitClusterClient;
use tonic::transport::Server;
use tracing::level_filters::LevelFilter;
use tracing::*;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod auth;
mod config;
mod logs_ingestor;
mod quickwit;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv();

    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_ansi(true)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");

    let config = IngestorConfig::load()?;
    let mut quickwit_clusters = vec![];

    info!("Creating Quickwit clients");
    for cluster in config.quickwit_clusters {
        quickwit_clusters.push(QuickwitClusterClient::new(cluster.indexers)?);
    }

    info!("Creating Authenticator");
    let authenticator = Authenticator::new(config.server_config.main_server_url)
        .context("Failed to build authenticator. Maybe the server URL is invalid ?")?;

    let logs_ingestor = logs_ingestor::LogsIngestorService::new(
        quickwit_clusters.clone(),
        authenticator.clone(),
        config.storage_config.storage_path,
    )?;

    info!(
        "Launching gRPC server on  0.0.0.0:{}",
        config.grpc_config.grpc_port
    );
    Server::builder()
        .add_service(
            opentelemetry::proto::collector::logs::v1::logs_service_server::LogsServiceServer::new(
                logs_ingestor,
            ),
        )
        .serve(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            config.grpc_config.grpc_port,
        )))
        .await?;

    Ok(())
}
