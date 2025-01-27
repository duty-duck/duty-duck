use super::*;
use chrono::{DateTime, Utc};

/// A task that was scheduled to run, but ran late.
/// The task has not yet started running, but is still inside the lateness window
#[derive(getset::CopyGetters, getset::Getters)]
pub struct LateTask {
    #[getset(get = "pub")]
    pub(super) base: TaskBase,
    /// In the context of a late task, the "next due at" field is the time at which the task was due to run
    /// BEFORE it transitioned to late (i.e. when it was in the due state)
    #[getset(get_copy = "pub")]
    pub(super) next_due_at: DateTime<Utc>,
    // late tasks have a cron schedule
    #[getset(get = "pub")]
    pub(super) cron_schedule: cron::Schedule,
}

impl LateTask {
    /// Checks whether the task should transition to the absent state
    /// (outside the lateness window)
    pub fn is_absent(&self, now: DateTime<Utc>) -> bool {
        now >= self.next_due_at + self.base.start_window + self.base.lateness_window
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
            next_due_at: self.next_due_at,
            cron_schedule: self.cron_schedule,
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
            // When a task starts, its next_due_at field is updated to the next time the task is due to run
            next_due_at: calculate_next_due_at(
                self.base.cron_schedule.as_ref(),
                self.base.schedule_timezone.as_ref(),
                now,
            )?,
            base: TaskBase {
                previous_status: Some(TaskStatus::Late),
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
            status: TaskStatus::Late,
            // Next due at is required for late tasks, and it's not present in the base task,
            // so we need to add it here
            next_due_at: Some(task.next_due_at),
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
        let next_due_at = boundary
            .next_due_at
            .ok_or(TaskError::FailedToBuildFromBoundary {
                details: "Next due at is required for late task".to_string(),
            })?;
        let base: TaskBase = boundary.try_into()?;
        let cron_schedule =
            base.cron_schedule
                .clone()
                .ok_or(TaskError::FailedToBuildFromBoundary {
                    details: "Cron schedule is required for late task".to_string(),
                })?;
        Ok(LateTask {
            next_due_at,
            cron_schedule,
            base,
        })
    }
}
