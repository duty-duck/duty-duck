use std::fmt::{self, Display};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema, Clone, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, TS, ToSchema, Clone)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    #[ts(type = "string")]
    pub id: TaskId,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// The current status of the task
    pub status: TaskStatus,
    /// The status of the task before the most recent status change
    pub previous_status: TaskStatus,
    /// The time at which the most recent status change occurred
    pub last_status_change_at: Option<DateTime<Utc>>,
    /// The cron schedule of the task (empty for non-cron tasks)
    pub cron_schedule: Option<String>,
    /// The time at which the task is next expected to run
    /// `None` for non-cron and disabled tasks
    pub next_due_at: Option<DateTime<Utc>>,
    /// Time before task is considered late
    pub start_window_seconds: i32,
    /// Time after task is considered late and before it is considered absent
    pub lateness_window_seconds: i32,
    /// Time after which task is considered dead without heartbeat
    pub heartbeat_timeout_seconds: i32,
    /// The time at which the task was created
    pub created_at: DateTime<Utc>,
}

impl Task {
    pub fn cron_schedule(&self) -> Option<croner::Cron> {
        croner::Cron::new(self.cron_schedule.as_ref()?).parse().ok()
    }
}

/// An enum that represents the status of a task run
#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum TaskStatus {
    /// The task is inactive and not scheduled to run
    Inactive = 0,
    /// The task is currently running
    Running = 1,
    /// The task is not yet expected to run
    Pending = 2,
    /// The task is expected to start soon (within the start window)
    Due = 3,
    /// The task is expected to start and is late (within the lateness window)
    Late = 4,
    /// The task is expected to start but has not started and the lateness window has passed
    Absent = 5,
    /// The task was running but is now presumed dead (no heartbeat within the heartbeat timeout)
    Dead = 6,
    /// The task was running but was aborted (e.g. by a user or system)
    /// Similar to Dead, but the transition was explicit, not due to heartbeat timeout
    Aborted = 7,
}

impl From<i16> for TaskStatus {
    fn from(value: i16) -> Self {
        match value {
            0 => Self::Inactive,
            1 => Self::Running,
            2 => Self::Pending,
            3 => Self::Due,
            4 => Self::Late,
            5 => Self::Absent,
            6 => Self::Dead,
            7 => Self::Aborted,
            _ => panic!("invalid TaskStatus discriminant: {value}"),
        }
    }
}

impl From<Option<i16>> for TaskStatus {
    fn from(value: Option<i16>) -> Self {
        value.map(|v| v.into()).unwrap_or(Self::Inactive)
    }
}
