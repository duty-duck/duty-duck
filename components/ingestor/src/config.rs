use envconfig::Envconfig;
use reqwest::Url;

#[derive(Envconfig, Debug, Clone)]
pub struct ServerConfig {
    #[envconfig(from = "GRPC_PORT", default = "50051")]
    pub grpc_port: u32,
}

#[derive(Debug, Clone)]
pub struct QuickwitCluster {
    pub name: String,
    pub indexers: Vec<Url>,
}

#[derive(Debug, Clone)]
pub struct IngestorConfig {
    pub quickwit_clusters: Vec<QuickwitCluster>,
    pub server_config: ServerConfig,
}

impl IngestorConfig {
    pub fn load() -> anyhow::Result<Self> {
        let server_config = ServerConfig::init_from_env()?;

        let quickwit_clusters = std::env::vars()
            .filter_map(|(key, value)| {
                let name = key.strip_prefix("QUICKWIT_CLUSTER_")?;
                let indexers = value
                    .split(",")
                    .filter_map(|v| Url::parse(v).ok())
                    .collect();
                Some(QuickwitCluster {
                    name: name.to_string(),
                    indexers,
                })
            })
            .collect::<Vec<_>>();

        if quickwit_clusters.is_empty() {
            anyhow::bail!("no quickwit cluster configuration. You must set the QUICKWIT_CLUSTER_<cluster name> env variable value to the urls of the indexers");
        }

        Ok(Self {
            server_config,
            quickwit_clusters,
        })
    }
}
