use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::QuickwitClient;

#[derive(Debug, Serialize)]
pub struct SearchRequest {
    /// Query text
    pub query: String,
    /// If set, restrict search to documents with a timestamp >= start_timestamp, taking advantage of potential time pruning oportunities. The value must be in seconds.
    pub start_timestamp: Option<i64>,
    /// If set, restrict search to documents with a timestamp < end_timestamp, taking advantage of potential time pruning oportunities. The value must be in seconds.
    pub end_timestamp: Option<i64>,
    /// Number of documents to skip
    pub start_offset: u64,
    /// Number of documents to return
    pub max_hits: u64,
    /// Fields to search on if no field name is specified in the query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_fields: Option<Vec<String>>,
    /// Fields to extract snippet on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet_fields: Option<Vec<String>>,
    /// Fields to sort the query results on. You can sort by one or two fast fields or by BM25 _score (requires fieldnorms).
    /// By default, hits are sorted by their document ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    /// Results of the query
    pub hits: Vec<serde_json::Map<String, serde_json::Value>>,
    /// Total number of matches
    pub num_hits: u64,
    /// Processing time of the query
    pub elapsed_time_micros: u64,
}

#[derive(Clone)]
pub struct SearchAPIV1 {
    pub(crate) client: QuickwitClient,
}

impl SearchAPIV1 {
    pub async fn search(
        &self,
        index_id: &str,
        request: &SearchRequest,
    ) -> anyhow::Result<SearchResponse> {
        let url = self
            .client
            .base_url
            .join(&format!("/api/v1/{index_id}/search"))?;
        let res = self.client.client.post(url).json(request).send().await?;

        let res_status = res.status();
        if res_status.is_client_error() {
            let body = res.text().await?;
            anyhow::bail!(
                "Client error occured when searching index, status = {}, body = {}",
                res_status.as_u16(),
                body
            );
        }

        res.error_for_status()
            .context("Search API: invalid HTTP status")?
            .json()
            .await
            .context("Search API: Failed to deserialize results")
    }
}
