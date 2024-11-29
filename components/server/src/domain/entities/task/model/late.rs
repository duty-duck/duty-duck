use super::*;
use chrono::{DateTime, Utc};

/// A task that was scheduled to run, but ran late.
/// The task has not yet started running, but is still inside the lateness window
#[derive(getset::Getters)]
pub struct LateTask {
    pub(super) base: TaskBase,
    pub(super) next_due_at: DateTime<Utc>,
}

impl LateTask {
    /// Checks whether the task should transition to the absent state
    /// (outside the lateness window)
    pub fn is_absent(&self, now: DateTime<Utc>) -> bool {
        now >= self.next_due_at + self.base.lateness_window
    }

    /// State transition: Late -> Absent
    pub fn mark_absent(self, now: DateTime<Utc>) -> Result<AbsentTask, TaskError> {
        if !self.is_absent(now) {
            return Err(TaskError::InvalidStateTransition {
                from: TaskStatus::Late,
                to: TaskStatus::Absent,
                details: "this task is not absent".to_string(),
            });
        }
        Ok(AbsentTask {
            next_due_at: calculate_next_due_at(&self.base.cron_schedule, now)?
                .ok_or(TaskError::InvalidCronSchedule)?,
            base: TaskBase {
                previous_status: Some(TaskStatus::Late),
                last_status_change_at: Some(Utc::now()),
                ..self.base
            },
        })
    }

    /// State transition: Late -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Due),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }
}


impl TryFrom<LateTask> for BoundaryTask {
    type Error = TaskError;

    fn try_from(task: LateTask) -> Result<Self, Self::Error> {
        Ok(BoundaryTask {
            status: TaskStatus::Absent,
            next_due_at: calculate_next_due_at(&task.base.cron_schedule, Utc::now())?,
            ..BoundaryTask::from(task.base)
        })
    }
}

impl TryFrom<BoundaryTask> for LateTask {
    type Error = TaskError;

    fn try_from(boundary: BoundaryTask) -> Result<Self, Self::Error> {
        if boundary.status != TaskStatus::Late {
            return Err(TaskError::FailedToBuildFromBoundary {
                details: "task status must be late".to_string(),
            });
        }
        Ok(LateTask {
            next_due_at: boundary
                .next_due_at
                .ok_or(TaskError::FailedToBuildFromBoundary {
                    details: "Next due at is required for late task".to_string(),
                })?,
            base: boundary.try_into()?,
        })
    }
}
