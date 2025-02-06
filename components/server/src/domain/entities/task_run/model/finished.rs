use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::super::boundary::{BoundaryTaskRun, TaskRunStatus};
use super::TaskRunError;
use crate::domain::entities::entity_metadata::EntityMetadata;

#[derive(Debug, Clone)]
pub struct FinishedTaskRun {
    pub(super) organization_id: Uuid,
    pub(super) id: Uuid,
    pub(super) task_id: Uuid,
    pub(super) started_at: DateTime<Utc>,
    pub(super) completed_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
    pub(super) exit_code: Option<i32>,
    pub(super) metadata: EntityMetadata,
}

impl TryFrom<BoundaryTaskRun> for FinishedTaskRun {
    type Error = TaskRunError;

    fn try_from(boundary: BoundaryTaskRun) -> Result<Self, Self::Error> {
        if boundary.status != TaskRunStatus::Finished {
            return Err(TaskRunError::FailedToBuildFromBoundary {
                details: "Task run status is not Finished".to_string(),
            });
        }

        let completed_at =
            boundary
                .completed_at
                .ok_or(TaskRunError::FailedToBuildFromBoundary {
                    details: "Finished task run must have completed_at".to_string(),
                })?;

        Ok(Self {
            organization_id: boundary.organization_id,
            id: boundary.id,
            task_id: boundary.task_id,
            started_at: boundary.started_at,
            completed_at,
            updated_at: boundary.updated_at,
            exit_code: boundary.exit_code,
            metadata: boundary.metadata,
        })
    }
}

impl From<FinishedTaskRun> for BoundaryTaskRun {
    fn from(finished: FinishedTaskRun) -> Self {
        Self {
            status: TaskRunStatus::Finished,
            organization_id: finished.organization_id,
            id: finished.id,
            task_id: finished.task_id,
            started_at: finished.started_at,
            updated_at: finished.updated_at,
            completed_at: Some(finished.completed_at),
            exit_code: finished.exit_code,
            error_message: None,
            last_heartbeat_at: None,
            heartbeat_timeout_seconds: 0,
            metadata: finished.metadata,
        }
    }
}
