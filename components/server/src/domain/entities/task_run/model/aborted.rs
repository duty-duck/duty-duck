use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::super::boundary::{BoundaryTaskRun, TaskRunStatus};
use super::TaskRunError;
use crate::domain::entities::entity_metadata::EntityMetadata;
use crate::domain::entities::task::TaskId;

#[derive(Debug, Clone)]
pub struct AbortedTaskRun {
    pub(super) organization_id: Uuid,
    pub(super) task_id: Uuid,
    pub(super) task_user_id: TaskId,
    pub(super) started_at: DateTime<Utc>,
    pub(super) completed_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
    pub(super) metadata: EntityMetadata,
}

impl TryFrom<BoundaryTaskRun> for AbortedTaskRun {
    type Error = TaskRunError;

    fn try_from(boundary: BoundaryTaskRun) -> Result<Self, Self::Error> {
        if boundary.status != TaskRunStatus::Aborted {
            return Err(TaskRunError::FailedToBuildFromBoundary {
                details: "Task run status is not Aborted".to_string(),
            });
        }

        let completed_at =
            boundary
                .completed_at
                .ok_or(TaskRunError::FailedToBuildFromBoundary {
                    details: "Aborted task run must have completed_at".to_string(),
                })?;

        Ok(Self {
            organization_id: boundary.organization_id,
            task_id: boundary.task_id,
            task_user_id: boundary.task_user_id,
            started_at: boundary.started_at,
            completed_at,
            updated_at: boundary.updated_at,
            metadata: boundary.metadata,
        })
    }
}

impl From<AbortedTaskRun> for BoundaryTaskRun {
    fn from(aborted: AbortedTaskRun) -> Self {
        Self {
            status: TaskRunStatus::Aborted,
            organization_id: aborted.organization_id,
            task_id: aborted.task_id,
            task_user_id: aborted.task_user_id,
            started_at: aborted.started_at,
            updated_at: aborted.updated_at,
            metadata: aborted.metadata,
            completed_at: Some(aborted.completed_at),
            exit_code: None,
            error_message: None,
            last_heartbeat_at: None,
            heartbeat_timeout_seconds: 0,
        }
    }
}
