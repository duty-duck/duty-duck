use crate::domain::entities::{task::{DueTaskAggregate, TaskAggregateError}, task_run::RunningTaskRun};

use super::*;

/// A task that is scheduled to run at a given time in the future
#[derive(getset::Getters)]
pub struct PendingTask {
    pub(super) base: TaskBase,
    pub(super) next_due_at: DateTime<Utc>,
}

impl PendingTask {
    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        now >= self.next_due_at
    }

    /// State transition: Pending -> Due
    pub fn mark_due(self, now: DateTime<Utc>) -> Result<DueTask, TaskError> {
        if !self.is_due(now) {
            return Err(TaskError::InvalidStateTransition {
                from: TaskStatus::Pending,
                to: TaskStatus::Due,
                details: "this task is not due to run yet".to_string(),
            });
        }
        Ok(DueTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Pending),
                last_status_change_at: Some(now),
                ..self.base
            },
            next_due_at: self.next_due_at,
        })
    }

    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Pending),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }
}


impl TryFrom<PendingTask> for BoundaryTask {
    type Error = TaskError;

    fn try_from(task: PendingTask) -> Result<Self, Self::Error> {
        Ok(BoundaryTask {
            status: TaskStatus::Absent,
            next_due_at: calculate_next_due_at(&task.base.cron_schedule, Utc::now())?,
            ..BoundaryTask::from(task.base)
        })
    }
}

impl TryFrom<BoundaryTask> for PendingTask {
    type Error = TaskError;

    fn try_from(boundary: BoundaryTask) -> Result<Self, Self::Error> {
        if boundary.status != TaskStatus::Pending {
            return Err(TaskError::FailedToBuildFromBoundary {
                details: "task status must be pending".to_string(),
            });
        }
        Ok(PendingTask {
            next_due_at: boundary
                .next_due_at
                .ok_or(TaskError::FailedToBuildFromBoundary {
                    details: "Next due at is required for pending task".to_string(),
                })?,
            base: boundary.try_into()?,
        })
    }
}
