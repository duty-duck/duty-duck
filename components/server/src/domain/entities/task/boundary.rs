use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::id::TaskId;

#[derive(Debug, Serialize, Deserialize, TS, ToSchema, Clone)]
#[ts(export)]
#[ts(rename = "Task")]
#[serde(rename_all = "camelCase")]
#[schema(as = Task)]
pub struct BoundaryTask {
    #[ts(type = "string")]
    pub id: TaskId,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub previous_status: Option<TaskStatus>,
    pub last_status_change_at: Option<DateTime<Utc>>,
    pub cron_schedule: Option<String>,
    pub next_due_at: Option<DateTime<Utc>>,
    pub start_window_seconds: i32,
    pub lateness_window_seconds: i32,
    pub heartbeat_timeout_seconds: i32,
    pub created_at: DateTime<Utc>,
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
    /// The latest task run was running but is now presumed dead (no heartbeat within the heartbeat timeout)
    Dead = 6,
    /// The latest task run was running but was aborted (e.g. by a user or system)
    /// Similar to Dead, but the transition was explicit, not due to heartbeat timeout
    Aborted = 7,
    /// The latest task run has finished running successfully
    Finished = 8,
    /// The latest task run has finished running unsuccessfully
    Failed = 9,
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
