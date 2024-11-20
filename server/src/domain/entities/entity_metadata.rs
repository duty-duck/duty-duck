use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;
use utoipa::ToSchema;
use tracing::error;

/// Key-value pairs of data that can be attached to monitors, incidents, etc. to add context
/// and filter entities
#[derive(Serialize, Deserialize, TS, Debug, Clone, ToSchema, Default)]
#[ts(export)]
pub struct EntityMetadata {
    pub records: HashMap<String, String>,
}

impl From<Option<Value>> for EntityMetadata {
    fn from(value: Option<Value>) -> Self {
        match value {
            Some(value) => match serde_json::from_value(value) {
                Ok(metadata) => metadata,
                Err(e) => {
                    error!("Error parsing entity metadata: {}", e);
                    EntityMetadata::default()
                }
            },
            None => EntityMetadata::default(),
        }
    }
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, ToSchema)]
#[ts(export)]
pub struct FilterableMetadataItem {
    pub key: String,
    pub distinct_values: Vec<FilterableMetadataValue>,
    pub key_cardinality: u64
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, ToSchema)]
#[ts(export)]
pub struct FilterableMetadataValue {
    pub value: String,
    pub value_count: u64
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, ToSchema)]
#[ts(export)]
pub struct FilterableMetadata {
    pub items: Vec<FilterableMetadataItem>,
}

/// An object used to filter database entities by metadata
/// It will match all rows if no filters are provided, otherwise,
/// it will match rows that have all the provided records, and for each record,
/// it will match rows that have any of the provided values
#[derive(Serialize, Deserialize, TS, Debug, Clone, ToSchema, Default)]
#[ts(export)]
pub struct MetadataFilter {
    pub items: HashMap<String, Vec<String>>,
}