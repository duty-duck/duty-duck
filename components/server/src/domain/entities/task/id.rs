use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum TaskId {
    // note that since we use the untagged JSON representation for this enum, the order of the variants is important:
    // we want to match the UUID variant first, because a UUID is also a valid string
    /// A UUID for a task, generated internally by the system
    Uuid(Uuid),
    /// A user-defined ID for a task
    UserId(TaskUserId),
}

/// A user-defined ID for a task
#[derive(Debug, Serialize, ToSchema, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct TaskUserId(String);

impl TaskUserId {
    pub fn new(id: String) -> Option<Self> {
        if id.is_empty() || id.contains(' ') {
            return None;
        }
        Some(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create a new user-defined task id from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid.to_string())
    }
}

/// A conversion from string to task id, only used by sqlx
impl From<String> for TaskUserId {
    fn from(value: String) -> Self {
        Self::new(value).expect("invalid task id")
    }
}

impl<'de> Deserialize<'de> for TaskUserId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::new(string).ok_or_else(|| serde::de::Error::custom("invalid task id"))
    }
}

impl Display for TaskUserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
