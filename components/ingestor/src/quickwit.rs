use std::sync::Arc;

use quickwit_client_rs::QuickwitClient;
use reqwest::Url;
use tracing::trace;

/// A client for a cluster of Quickwit indexers, designed to partition incoming requests so that a logs ingestion request for a given request
/// goes to a specific indexer in the cluster.
#[derive(Clone)]
pub struct QuickwitClusterClient {
    subclients: Arc<Vec<QuickwitClient>>,
}

impl QuickwitClusterClient {
    pub fn new(mut urls: Vec<Url>) -> anyhow::Result<Self> {
        // sort urls to ensure `get_client` is deterministic (given the same set of urls)
        urls.sort();

        let mut subclients = vec![];
        for url in urls {
            subclients.push(QuickwitClient::new(url)?);
        }
        Ok(Self {
            subclients: Arc::new(subclients),
        })
    }

    pub fn get_any_client(&self) -> &QuickwitClient {
        &self.subclients[0]
    }

    /// Gets a reference to a Quickwit client for a given indexer, based on a partition key
    /// Indexers are partitioned so a given partition key always maps to the same indexer
    pub fn get_client(&self, partition_key: &str) -> &QuickwitClient {
        let client_index =
            crc32fast::hash(partition_key.as_bytes()) as usize % self.subclients.len();
        let client = &self.subclients[client_index];
        trace!(
            ?partition_key,
            indexer_url = client.base_url().as_str(),
            "Quickwit indexer for partition key {} is {}",
            partition_key,
            client.base_url()
        );

        client
    }
}
