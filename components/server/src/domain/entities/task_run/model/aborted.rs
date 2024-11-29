use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::entities::task::TaskId;
use super::TaskRunError;
use super::super::boundary::{BoundaryTaskRun, TaskRunStatus};

pub struct AbortedTaskRun {
    organization_id: Uuid,
    task_id: TaskId,
    started_at: DateTime<Utc>,
    completed_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<BoundaryTaskRun> for AbortedTaskRun {
    type Error = TaskRunError;

    fn try_from(boundary: BoundaryTaskRun) -> Result<Self, Self::Error> {
        if boundary.status != TaskRunStatus::Aborted {
            return Err(TaskRunError::FailedToBuildFromBoundary { 
                details: "Task run status is not Aborted".to_string() 
            });
        }

        let completed_at = boundary.completed_at.ok_or(
            TaskRunError::FailedToBuildFromBoundary { 
                details: "Aborted task run must have completed_at".to_string() 
            })?;

        Ok(Self {
            organization_id: boundary.organization_id,
            task_id: boundary.task_id,
            started_at: boundary.started_at,
            completed_at,
            updated_at: boundary.updated_at,
        })
    }
}

impl From<AbortedTaskRun> for BoundaryTaskRun {
    fn from(aborted: AbortedTaskRun) -> Self {
        Self {
            status: TaskRunStatus::Aborted,
            organization_id: aborted.organization_id,
            task_id: aborted.task_id,
            started_at: aborted.started_at,
            updated_at: aborted.updated_at,
            completed_at: Some(aborted.completed_at),
            exit_code: None,
            error_message: None,
            last_heartbeat_at: None
        }
    }
}