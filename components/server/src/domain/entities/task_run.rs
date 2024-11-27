use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::task::TaskId;

#[derive(Debug, Serialize, Deserialize, TS, ToSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TaskRun {
    pub organization_id: Uuid,
    #[ts(type = "string")]
    pub task_id: TaskId,
    pub status: TaskRunStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub error_message: Option<String>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
}

/// An enum that represents the status of a task run
#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum TaskRunStatus {
    /// The task run is currently running
    Running = 1,
    /// The task run has completed successfully
    Completed = 2,
    /// The task run has completed with an error
    Failed = 3,
    /// The task run was aborted (e.g. by a user or system)
    Aborted = 4,
    /// The task run was presumed dead (no heartbeat within the heartbeat timeout)
    /// but it may still be running
    Dead = 5,
}

impl From<i16> for TaskRunStatus {
    fn from(value: i16) -> Self {
        match value {
            1 => Self::Running,
            2 => Self::Completed,
            3 => Self::Failed,
            4 => Self::Aborted,
            5 => Self::Dead,
            _ => panic!("invalid TaskRunStatus discriminant: {value}"),
        }
    }
}
