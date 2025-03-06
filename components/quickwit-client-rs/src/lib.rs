use std::sync::Arc;

use anyhow::Context;
use indexes_api_v1::IndexesAPIV1;
use ingest_api_v1::IngestAPIV1;
use reqwest::{IntoUrl, Url};
use search_api_v1::SearchAPIV1;

pub mod indexes_api_v1;
pub mod ingest_api_v1;
pub mod search_api_v1;

#[derive(Clone)]
pub struct QuickwitClient {
    pub(crate) client: reqwest::Client,
    base_url: Arc<Url>,
}

impl QuickwitClient {
    pub fn new(api_url: impl IntoUrl) -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: Arc::new(
                api_url
                    .into_url()
                    .context("invalid API URL for Quickwit client")?,
            ),
        })
    }

    pub fn ingest_api_v1(&self) -> IngestAPIV1 {
        IngestAPIV1 {
            client: self.clone(),
        }
    }

    pub fn indexes_api_v1(&self) -> IndexesAPIV1 {
        IndexesAPIV1 {
            client: self.clone(),
        }
    }

    pub fn search_api_v1(&self) -> SearchAPIV1 {
        SearchAPIV1 {
            client: self.clone(),
        }
    }
}
