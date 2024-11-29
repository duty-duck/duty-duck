use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::entities::task::TaskId;
use super::TaskRunError;
use super::super::boundary::{BoundaryTaskRun, TaskRunStatus};

pub struct FailedTaskRun {
    pub(super) organization_id: Uuid,
    pub(super) task_id: TaskId,
    pub(super) started_at: DateTime<Utc>,
    pub(super) completed_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
    pub(super) exit_code: Option<i32>,
    pub(super) error_message: Option<String>,
}


impl TryFrom<BoundaryTaskRun> for FailedTaskRun {
    type Error = TaskRunError;

    fn try_from(boundary: BoundaryTaskRun) -> Result<Self, Self::Error> {
        if boundary.status != TaskRunStatus::Failed {
            return Err(TaskRunError::FailedToBuildFromBoundary { 
                details: "Task run status is not Failed".to_string() 
            });
        }

        let completed_at = boundary.completed_at.ok_or(
            TaskRunError::FailedToBuildFromBoundary { 
                details: "Failed task run must have completed_at".to_string() 
            })?;

        Ok(Self {
            organization_id: boundary.organization_id,
            task_id: boundary.task_id,
            started_at: boundary.started_at,
            completed_at,
            updated_at: boundary.updated_at,
            exit_code: boundary.exit_code,
            error_message: boundary.error_message,
        })
    }
}

impl From<FailedTaskRun> for BoundaryTaskRun {
    fn from(failed: FailedTaskRun) -> Self {
        Self {
            status: TaskRunStatus::Failed,
            organization_id: failed.organization_id,
            task_id: failed.task_id,
            started_at: failed.started_at,
            updated_at: failed.updated_at,
            completed_at: Some(failed.completed_at),
            exit_code: failed.exit_code,
            error_message: failed.error_message,
            last_heartbeat_at: None,
        }
    }
}