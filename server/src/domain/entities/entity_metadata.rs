use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;
use utoipa::ToSchema;

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
