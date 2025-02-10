use super::*;

/// A task that was running, and finished unsuccessfully
#[derive(getset::Getters, Debug)]
#[getset(get = "pub")]
pub struct FailingTask {
    /// The next time the task is due to run
    pub(super) next_due_at: Option<DateTime<Utc>>,
    pub(super) base: TaskBase,
}

impl FailingTask {
    /// State transition: Failing -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            // When a task starts, its next_due_at field is updated to the next time the task is due to run
            next_due_at: calculate_next_due_at(
                self.base.cron_schedule.as_ref(),
                self.base.schedule_timezone.as_ref(),
                now,
            )?,
            base: TaskBase {
                previous_status: Some(TaskStatus::Failing),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }

    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        self.next_due_at.is_some_and(|due_at| now >= due_at)
    }

    /// State transition: Failing -> Due
    pub fn mark_due(self, now: DateTime<Utc>) -> Result<DueTask, TaskError> {
        if self.base.cron_schedule.is_none() {
            return Err(TaskError::InvalidStateTransition {
                from: TaskStatus::Failing,
                to: TaskStatus::Due,
                details: "this task is not scheduled to run, it has no cron schedule".to_string(),
            });
        }
        if !self.is_due(now) {
            return Err(TaskError::InvalidStateTransition {
                from: TaskStatus::Failing,
                to: TaskStatus::Due,
                details: "this task has a cron schedule but is not due to run yet".to_string(),
            });
        }
        Ok(DueTask {
            // unwrap is safe because we already checked that the task is due to run,
            // so it must have a next_due_at
            next_due_at: self.next_due_at.unwrap(),
            cron_schedule: self.base.cron_schedule.clone().unwrap(),
            base: TaskBase {
                previous_status: Some(TaskStatus::Failing),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }

    /// State transition: Failing -> Archived
    pub fn archive(self, now: DateTime<Utc>) -> ArchivedTask {
        ArchivedTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Failing),
                last_status_change_at: Some(now),
                ..self.base
            },
        }
    }
}

impl TryFrom<FailingTask> for BoundaryTask {
    type Error = TaskError;

    fn try_from(task: FailingTask) -> Result<Self, Self::Error> {
        Ok(BoundaryTask {
            status: TaskStatus::Failing,
            next_due_at: task.next_due_at,
            ..BoundaryTask::from(task.base)
        })
    }
}

impl TryFrom<BoundaryTask> for FailingTask {
    type Error = TaskError;

    fn try_from(boundary: BoundaryTask) -> Result<Self, Self::Error> {
        if boundary.status != TaskStatus::Failing {
            return Err(TaskError::FailedToBuildFromBoundary {
                details: "task status must be failing".to_string(),
            });
        }
        Ok(FailingTask {
            next_due_at: boundary.next_due_at,
            base: boundary.try_into()?,
        })
    }
}
