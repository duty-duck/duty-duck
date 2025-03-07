use anyhow::Context;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

pub mod otel;

use crate::QuickwitClient;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Version {
    #[serde(rename = "0.8")]
    V08,
}

#[derive(Debug, Serialize)]
pub struct IndexConfig {
    pub index_id: String,
    pub version: Version,
    pub doc_mapping: DocMapping,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention: Option<RetentionSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_settings: Option<SearchSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexing_settings: Option<IndexingSettings>,
}

#[derive(Debug, Serialize)]
pub struct RetentionSettings {
    /// Duration of time for which the splits should be retained, expressed in a human-friendly way (1 hour, 3 days, a week, ...).
    pub period: String,
    /// Defines the frequency at which the retention policy is evaluated and applied, expressed in a human-friendly way (hourly, daily, ...) or as a cron expression (0 0 * * * *, 0 0 0 * * *).
    pub schedule: String,
}

#[derive(Debug, Serialize)]
pub struct IndexingSettings {
    /// Default is 60
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_timeout_secs: Option<u64>,
    /// Default is 1000000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docstore_blocksize: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<IndexingResources>,
}

#[derive(Debug, Serialize)]
pub struct IndexingResources {
    /// Defaults to "2 GB"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heap_size: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_search_fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DocMappingMode {
    Dynamic,
    Strict,
    Lenient,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Record {
    Basic,
    Freq,
    Position,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FieldMapping {
    Text(TextFieldMapping),
    #[serde(rename = "u64")]
    Unsigned64(NumericFieldMapping),
    #[serde(rename = "i64")]
    Signed64(NumericFieldMapping),
    #[serde(rename = "f64")]
    Float64(NumericFieldMapping),
    DateTime(DateTimeFieldMapping),
    Bool(BoolFieldMapping),
    Bytes(BytesFieldMapping),
    Json(JsonFieldMapping),
}

#[derive(Debug, Serialize)]
pub struct TextFieldMapping {
    /// Name for the field
    pub name: String,
    /// Optional description for the field.
    pub description: Option<String>,
    /// Whether value is stored in the document store
    pub stored: bool,
    /// Whether value should be indexed so it can be searched
    pub indexed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record: Option<Record>,
    /// Whether to store fieldnorms for the field. Fieldnorms are required to calculate the BM25 Score of the document. (default is false)
    pub fieldnorms: bool,
    /// Whether value is stored in a fast field. The fast field will contain the term ids and the dictionary (default is false)
    pub fast: bool,
}

#[derive(Debug, Serialize)]
pub struct NumericFieldMapping {
    /// Name for the field
    pub name: String,
    /// Optional description for the field.
    pub description: Option<String>,
    /// Whether value is stored in the document store
    pub stored: bool,
    /// Whether value should be indexed so it can be searched
    pub indexed: bool,
    /// Whether value is stored in a fast field. The fast field will contain the term ids and the dictionary (default is false)
    pub fast: bool,
    /// Whether to convert numbers passed as strings to integers or floats. (Default is true)
    pub coerce: bool,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum DateTimeFastPrecision {
    Seconds,
    MilliSeconds,
    MicroSeconds,
    NanoSeconds,
}

#[derive(Debug, Serialize)]
pub struct DateTimeFieldMapping {
    /// Name for the field
    pub name: String,
    /// Optional description for the field.
    pub description: Option<String>,
    /// Whether value is stored in the document store
    pub stored: bool,
    /// Whether value should be indexed so it can be searched
    pub indexed: bool,
    /// Whether value is stored in a fast field. The fast field will contain the term ids and the dictionary (default is false)
    pub fast: bool,
    /// Formats used to parse input dates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_formats: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    /// The precision (seconds, milliseconds, microseconds, or nanoseconds) used to store the fast values. (default is seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fast_precision: Option<DateTimeFastPrecision>,
}

#[derive(Debug, Serialize)]
pub struct BoolFieldMapping {
    /// Name for the field
    pub name: String,
    /// Optional description for the field.
    pub description: Option<String>,
    /// Whether value is stored in the document store
    pub stored: bool,
    /// Whether value should be indexed so it can be searched
    pub indexed: bool,
    /// Whether value is stored in a fast field. The fast field will contain the term ids and the dictionary (default is false)
    pub fast: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BytesEncoding {
    Hex,
    Base64,
}

#[derive(Debug, Serialize)]
pub struct BytesFieldMapping {
    /// Name for the field
    pub name: String,
    /// Optional description for the field.
    pub description: Option<String>,
    /// Whether value is stored in the document store
    pub stored: bool,
    /// Whether value should be indexed so it can be searched
    pub indexed: bool,
    /// Whether value is stored in a fast field. The fast field will contain the term ids and the dictionary (default is false)
    pub fast: bool,
    pub input_format: BytesEncoding,
    pub output_format: BytesEncoding,
}

#[derive(Debug, Serialize)]
pub struct JsonFieldMapping {
    /// Name for the field
    pub name: String,
    /// Optional description for the field.
    pub description: Option<String>,
    /// Whether value is stored in the document store
    pub stored: bool,
    /// Whether value should be indexed so it can be searched
    pub indexed: bool,
    /// Whether value is stored in a fast field. The fast field will contain the term ids and the dictionary (default is false)
    pub fast: bool,

    /// Only affects strings in the json object. Name of the Tokenizer, choices between raw, default, en_stem and chinese_compatible
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,
    /// Only affects strings in the json object. Describes the amount of information indexed, choices between basic, freq and position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record: Option<Record>,
    /// If true, json keys containing a . should be expanded.
    /// For instance, if expand_dots is set to true, {"k8s.node.id": "node-2"} will be indexed as if it was {"k8s": {"node": {"id": "node2"}}}
    pub expand_dots: bool,
}

#[derive(Debug, Serialize)]
pub struct DocMapping {
    pub mode: DocMappingMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_fields: Option<Vec<String>>,
    /// Whether or not the original JSON document is stored or not in the index.
    pub store_source: bool,
    /// If set, quickwit will route documents into different splits depending on the field name declared as the partition_key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_key: Option<String>,
    /// Limits the number of splits created through partitioning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_num_partitions: Option<usize>,
    pub field_mappings: Vec<FieldMapping>,
}

#[derive(Debug, Deserialize)]
pub struct DescribeIndexResponse {
    pub description: Option<String>,
    pub index_id: String,
    pub index_uri: String,
    pub min_timestamp: Option<u64>,
    pub max_timestamp: Option<u64>,
    pub num_published_docs: u64,
    pub num_published_splits: u64,
    pub size_published_docs_uncompressed: u64,
    pub size_published_splits: u64,
    pub timestamp_field_name: Option<String>,
}

#[derive(Clone)]
pub struct IndexesAPIV1 {
    pub(crate) client: QuickwitClient,
}

impl IndexesAPIV1 {
    pub async fn create_index(&self, config: &IndexConfig) -> anyhow::Result<()> {
        let url = self.client.base_url.join("/api/v1/indexes")?;
        let res = self.client.client.post(url).json(config).send().await?;
        let res_status = res.status();
        if res_status.is_client_error() {
            let body = res.text().await?;
            anyhow::bail!(
                "Client error occured when creating index, status = {}, body = {}",
                res_status.as_u16(),
                body
            );
        }

        res.error_for_status()?;
        Ok(())
    }

    pub async fn delete_index(&self, index_id: &str) -> anyhow::Result<()> {
        let url = self
            .client
            .base_url
            .join(&format!("/api/v1/indexes/{index_id}"))?;
        self.client
            .client
            .delete(url)
            .send()
            .await?
            .error_for_status()
            .context("Delete index invaild HTTP status")?;
        Ok(())
    }

    pub async fn describe_index(
        &self,
        index_id: &str,
    ) -> anyhow::Result<Option<DescribeIndexResponse>> {
        let url = self
            .client
            .base_url
            .join(&format!("/api/v1/indexes/{index_id}/describe"))?;
        let res = self.client.client.get(url).send().await?;

        if res.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            let res = res
                .error_for_status()
                .context("Clear index invaild HTTP status")?
                .json()
                .await
                .context("Failed to deserialize describe index response")?;

            Ok(Some(res))
        }
    }

    pub async fn clear_index(&self, index_id: &str) -> anyhow::Result<()> {
        let url = self
            .client
            .base_url
            .join(&format!("/api/v1/indexes/{index_id}/clear"))?;
        self.client
            .client
            .put(url)
            .send()
            .await?
            .error_for_status()
            .context("Clear index invaild HTTP status")?;
        Ok(())
    }
}
