use envconfig::Envconfig;
use reqwest::Url;
use tracing::*;

#[derive(Envconfig, Debug, Clone)]
pub struct ServerConfig {
    #[envconfig(from = "MAIN_SERVER_URL")]
    pub main_server_url: String,
}

#[derive(Envconfig, Debug, Clone)]
pub struct GrpcConfig {
    #[envconfig(from = "GRPC_PORT", default = "50051")]
    pub grpc_port: u16,
}

#[derive(Debug, Clone)]
pub struct QuickwitCluster {
    pub name: String,
    pub indexers: Vec<Url>,
}

#[derive(Debug, Clone)]
pub struct IngestorConfig {
    pub quickwit_clusters: Vec<QuickwitCluster>,
    pub grpc_config: GrpcConfig,
    pub server_config: ServerConfig,
}

impl IngestorConfig {
    pub fn load() -> anyhow::Result<Self> {
        debug!("Loading config from environment variables");

        let grpc_config = GrpcConfig::init_from_env()?;
        let server_config = ServerConfig::init_from_env()?;

        let mut quickwit_clusters = std::env::vars()
            .filter_map(|(key, value)| {
                let name = key.strip_prefix("QUICKWIT_CLUSTER_")?;
                let indexers = value
                    .split(",")
                    .filter_map(|v| Url::parse(v).ok())
                    .collect();

                info!(cluster_name = ?name, indexers = ?indexers, "Loaded config for Quickwit cluster '{}'", name);

                Some(QuickwitCluster {
                    name: name.to_string(),
                    indexers,
                })
            })
            .collect::<Vec<_>>();

        if quickwit_clusters.is_empty() {
            anyhow::bail!("no quickwit cluster configuration. You must set the QUICKWIT_CLUSTER_<cluster name> env variable value to the urls of the indexers");
        }

        // sort clusters by name
        quickwit_clusters.sort_by_key(|c| c.name.clone());

        info!("Loaded configuration from environment variables");

        Ok(Self {
            grpc_config,
            server_config,
            quickwit_clusters,
        })
    }
}
