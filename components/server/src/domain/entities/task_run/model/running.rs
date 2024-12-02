
use uuid::Uuid;
use chrono::{DateTime, Utc};
use getset::Getters;

use crate::domain::entities::task::TaskId;
use super::{DeadTaskRun, FailedTaskRun, FinishedTaskRun, TaskRunError};
use super::super::boundary::{BoundaryTaskRun, TaskRunStatus};

#[derive(Getters, Debug, Clone)]
#[getset(get = "pub")]
pub struct RunningTaskRun {
    organization_id: Uuid,
    task_id: TaskId,
    started_at: DateTime<Utc>,
    last_heartbeat_at: DateTime<Utc>,
}

impl RunningTaskRun {
    pub fn new(organization_id: Uuid, task_id: TaskId, started_at: DateTime<Utc>) -> Self {
        Self {
            organization_id,
            task_id,
            started_at,
            last_heartbeat_at: started_at,
        }
    }

    pub fn receive_heartbeat(self, now: DateTime<Utc>) -> Result<RunningTaskRun, TaskRunError> {
        Ok(RunningTaskRun {
            last_heartbeat_at: now,
            ..self
        })
    }

    /// Transition : Running -> Dead
    pub fn mark_dead(self, now: DateTime<Utc>) -> Result<DeadTaskRun, TaskRunError> {
        Ok(DeadTaskRun {
            organization_id: self.organization_id,
            task_id: self.task_id,
            started_at: self.started_at,
            completed_at: now,
            updated_at: now,
            last_heartbeat_at: self.last_heartbeat_at,
        })
    }

    /// Transition : Running -> Finished
    pub fn mark_finished(
        self,
        now: DateTime<Utc>,
        exit_code: Option<i32>,
    ) -> Result<FinishedTaskRun, TaskRunError> {
        if exit_code.is_some_and(|e| e > 0) {
            return Err(TaskRunError::InvalidStateTransition {
                from: TaskRunStatus::Running,
                to: TaskRunStatus::Finished,
                details: "exit code for a successful task run cannot be > 0".to_string(),
            });
        }
        Ok(FinishedTaskRun {
            organization_id: self.organization_id,
            task_id: self.task_id,
            started_at: self.started_at,
            completed_at: now,
            updated_at: now,
            exit_code,
        })
    }

    /// Transition : Running -> Failed
    pub fn mark_failed(
        self,
        now: DateTime<Utc>,
        exit_code: Option<i32>,
        error_message: Option<String>,
    ) -> Result<FailedTaskRun, TaskRunError> {
        if exit_code.is_some_and(|e| e == 0) {
            return Err(TaskRunError::InvalidStateTransition {
                from: TaskRunStatus::Running,
                to: TaskRunStatus::Failed,
                details: "exit code for a failed task run cannot be <= 0".to_string(),
            });
        }
        Ok(FailedTaskRun {
            organization_id: self.organization_id,
            task_id: self.task_id,
            started_at: self.started_at,
            completed_at: now,
            updated_at: now,
            exit_code,
            error_message,
        })
    }
}

impl TryFrom<BoundaryTaskRun> for RunningTaskRun {
    type Error = TaskRunError;

    fn try_from(boundary: BoundaryTaskRun) -> Result<Self, Self::Error> {
        if boundary.status != TaskRunStatus::Running {
            return Err(TaskRunError::FailedToBuildFromBoundary { 
                details: "Task run status is not Running".to_string() 
            });
        }

        let last_heartbeat_at = boundary.last_heartbeat_at.ok_or(
            TaskRunError::FailedToBuildFromBoundary { 
                details: "Running task run must have last_heartbeat_at".to_string() 
            })?;

        Ok(Self {
            organization_id: boundary.organization_id,
            task_id: boundary.task_id,
            started_at: boundary.started_at,
            last_heartbeat_at,
        })
    }
}

impl From<RunningTaskRun> for BoundaryTaskRun {
    fn from(running: RunningTaskRun) -> Self {
        Self {
            status: TaskRunStatus::Running,
            organization_id: running.organization_id,
            task_id: running.task_id,
            started_at: running.started_at,
            updated_at: running.last_heartbeat_at,
            completed_at: None,
            exit_code: None,
            error_message: None,
            last_heartbeat_at: Some(running.last_heartbeat_at),
        }
    }
}