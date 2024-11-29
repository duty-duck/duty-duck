///! This module defines the different states a task run can be in using separate structs
/// Together, these structs form a finite state machine that encompasses all possible states and all legal transitions between them.
/// 
/// To persist this state machine or serve it over the network, we use the `BoundaryTaskRun` type.
use chrono::{DateTime, Utc};
use uuid::Uuid;
use thiserror::Error;

use crate::domain::entities::task::TaskId;

use super::TaskRunStatus;

#[derive(Debug, Error)]
pub enum TaskRunError {
    #[error("Invalid state transition from {from:?} to {to:?}: {details}")]
    InvalidStateTransition {
        from: TaskRunStatus,
        to: TaskRunStatus,
        details: String,
    },
}

pub struct RunningTaskRun {
    pub organization_id: Uuid,
    pub task_id: TaskId,
    pub started_at: DateTime<Utc>,
    pub last_heartbeat_at: DateTime<Utc>,
}

impl RunningTaskRun {
    pub fn receive_heartbeat(self) -> Result<RunningTaskRun, TaskRunError> {
        Ok(RunningTaskRun {
            last_heartbeat_at: Utc::now(),
            ..self
        })
    }

    /// Transition : Running -> Dead
    pub fn mark_dead(self) -> Result<DeadTaskRun, TaskRunError> {
        let now = Utc::now();
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
    pub fn mark_finished(self, exit_code: Option<i32>) -> Result<FinishedTaskRun, TaskRunError> {
        if exit_code.is_some_and(|e| e > 0) {
            return Err(TaskRunError::InvalidStateTransition {
                from: TaskRunStatus::Running,
                to: TaskRunStatus::Completed,
                details: "exit code for a successful task run cannot be > 0".to_string(),
            });
        }
        Ok(FinishedTaskRun {
            organization_id: self.organization_id,
            task_id: self.task_id,
            started_at: self.started_at,
            completed_at: Utc::now(),
            exit_code,
        })
    }

    /// Transition : Running -> Failed
    pub fn mark_failed(self, exit_code: Option<i32>, error_message: Option<String>) -> Result<FailedTaskRun, TaskRunError> {
        if exit_code.is_some_and(|e| e <= 0) {
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
            completed_at: Utc::now(),
            exit_code,
            error_message,
        })
    }
}

pub struct FinishedTaskRun {
    pub organization_id: Uuid,
    pub task_id: TaskId,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub exit_code: Option<i32>,
}

pub struct FailedTaskRun {
    pub organization_id: Uuid,
    pub task_id: TaskId,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub error_message: Option<String>,
}

pub struct AbortedTaskRun {
    pub organization_id: Uuid,
    pub task_id: TaskId,
    pub started_at: DateTime<Utc>,
}

pub struct DeadTaskRun {
    pub organization_id: Uuid,
    pub task_id: TaskId,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_heartbeat_at: DateTime<Utc>,
}
