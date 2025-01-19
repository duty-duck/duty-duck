use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct TaskId(String);

impl TaskId {
    pub fn new(id: String) -> Option<Self> {
        if id.is_empty() || id.contains(' ') {
            return None;
        }
        Some(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A conversion from string to task id, only used by sqlx
impl From<String> for TaskId {
    fn from(value: String) -> Self {
        Self::new(value).expect("invalid task id")
    }
}

impl<'de> Deserialize<'de> for TaskId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::new(string).ok_or_else(|| serde::de::Error::custom("invalid task id"))
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
