use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::entities::task::TaskId;

use super::model::*;

/// A unspecialized representation of a task run, used at API and database boundaries
/// We have a set of conversions to/from this type to the specific task run types.
#[derive(Debug, Serialize, Deserialize, TS, ToSchema, Clone)]
#[ts(export)]
#[ts(rename = "TaskRun")]
#[serde(rename_all = "camelCase")]
#[schema(as = TaskRun)]
pub struct BoundaryTaskRun {
    pub organization_id: Uuid,
    #[ts(type = "string")]
    pub task_id: TaskId,
    pub status: TaskRunStatus,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

// Conversion implementations from/to boundary types from/to model types
impl TryFrom<BoundaryTaskRun> for RunningTaskRun {
    type Error = &'static str;

    fn try_from(boundary: BoundaryTaskRun) -> Result<Self, Self::Error> {
        if boundary.status != TaskRunStatus::Running {
            return Err("Task run is not in running state");
        }

        Ok(Self {
            organization_id: boundary.organization_id,
            task_id: boundary.task_id,
            started_at: boundary.started_at,
            last_heartbeat_at: boundary.last_heartbeat_at
                .ok_or("Running task run must have last_heartbeat_at")?,
        })
    }
}

impl From<RunningTaskRun> for BoundaryTaskRun {
    fn from(running: RunningTaskRun) -> Self {
        Self {
            organization_id: running.organization_id,
            task_id: running.task_id,
            status: TaskRunStatus::Running,
            started_at: running.started_at,
            updated_at: Utc::now(),
            completed_at: None,
            exit_code: None,
            error_message: None,
            last_heartbeat_at: Some(running.last_heartbeat_at),
        }
    }
}

impl TryFrom<BoundaryTaskRun> for FinishedTaskRun {
    type Error = &'static str;

    fn try_from(boundary: BoundaryTaskRun) -> Result<Self, Self::Error> {
        if boundary.status != TaskRunStatus::Completed {
            return Err("Task run is not in finished state");
        }

        Ok(Self {
            organization_id: boundary.organization_id,
            task_id: boundary.task_id,
            started_at: boundary.started_at,
            completed_at: boundary.completed_at
                .ok_or("Finished task run must have completed_at")?,
            exit_code: boundary.exit_code,
        })
    }
}

impl From<FinishedTaskRun> for BoundaryTaskRun {
    fn from(finished: FinishedTaskRun) -> Self {
        Self {
            organization_id: finished.organization_id,
            task_id: finished.task_id,
            status: TaskRunStatus::Completed,
            started_at: finished.started_at,
            updated_at: finished.completed_at,
            completed_at: Some(finished.completed_at),
            exit_code: finished.exit_code,
            error_message: None,
            last_heartbeat_at: None,
        }
    }
}

impl TryFrom<BoundaryTaskRun> for FailedTaskRun {
    type Error = &'static str;

    fn try_from(boundary: BoundaryTaskRun) -> Result<Self, Self::Error> {
        if boundary.status != TaskRunStatus::Failed {
            return Err("Task run is not in failed state");
        }

        Ok(Self {
            organization_id: boundary.organization_id,
            task_id: boundary.task_id,
            started_at: boundary.started_at,
            completed_at: boundary.completed_at
                .ok_or("Failed task run must have completed_at")?,
            error_message: boundary.error_message,
            exit_code: boundary.exit_code,
        })
    }
}

impl From<FailedTaskRun> for BoundaryTaskRun {
    fn from(failed: FailedTaskRun) -> Self {
        Self {
            organization_id: failed.organization_id,
            task_id: failed.task_id,
            status: TaskRunStatus::Failed,
            started_at: failed.started_at,
            updated_at: failed.completed_at,
            completed_at: Some(failed.completed_at),
            exit_code: failed.exit_code,
            error_message: failed.error_message,
            last_heartbeat_at: None,
        }
    }
}

impl TryFrom<BoundaryTaskRun> for AbortedTaskRun {
    type Error = &'static str;

    fn try_from(boundary: BoundaryTaskRun) -> Result<Self, Self::Error> {
        if boundary.status != TaskRunStatus::Aborted {
            return Err("Task run is not in aborted state");
        }

        Ok(Self {
            organization_id: boundary.organization_id,
            task_id: boundary.task_id,
            started_at: boundary.started_at,
        })
    }
}

impl From<AbortedTaskRun> for BoundaryTaskRun {
    fn from(aborted: AbortedTaskRun) -> Self {
        Self {
            organization_id: aborted.organization_id,
            task_id: aborted.task_id,
            status: TaskRunStatus::Aborted,
            started_at: aborted.started_at,
            updated_at: Utc::now(),
            completed_at: None,
            exit_code: None,
            error_message: None,
            last_heartbeat_at: None,
        }
    }
}
