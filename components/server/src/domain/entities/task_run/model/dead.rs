use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::entities::task::TaskId;
use super::TaskRunError;
use super::super::boundary::{BoundaryTaskRun, TaskRunStatus};

pub struct DeadTaskRun {
    pub(super) organization_id: Uuid,
    pub(super) task_id: TaskId,
    pub(super) started_at: DateTime<Utc>,
    pub(super) completed_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
    pub(super) last_heartbeat_at: DateTime<Utc>,
}

impl TryFrom<BoundaryTaskRun> for DeadTaskRun {
    type Error = TaskRunError;

    fn try_from(boundary: BoundaryTaskRun) -> Result<Self, Self::Error> {
        if boundary.status != TaskRunStatus::Dead {
            return Err(TaskRunError::FailedToBuildFromBoundary { 
                details: "Task run status is not Dead".to_string() 
            });
        }

        let completed_at = boundary.completed_at.ok_or(
            TaskRunError::FailedToBuildFromBoundary { 
                details: "Dead task run must have completed_at".to_string() 
            })?;

        let last_heartbeat_at = boundary.last_heartbeat_at.ok_or(
            TaskRunError::FailedToBuildFromBoundary { 
                details: "Dead task run must have last_heartbeat_at".to_string() 
            })?;

        Ok(Self {
            organization_id: boundary.organization_id,
            task_id: boundary.task_id,
            started_at: boundary.started_at,
            completed_at,
            updated_at: boundary.updated_at,
            last_heartbeat_at,
        })
    }
}


impl From<DeadTaskRun> for BoundaryTaskRun {
    fn from(dead: DeadTaskRun) -> Self {
        Self {
            status: TaskRunStatus::Dead,
            organization_id: dead.organization_id,
            task_id: dead.task_id,
            started_at: dead.started_at,
            updated_at: dead.updated_at,
            completed_at: Some(dead.completed_at),
            exit_code: None,
            error_message: None,
            last_heartbeat_at: None,
        }
    }
}
