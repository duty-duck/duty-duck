use super::*;

/// A task that whose scheduled time has come and is expected to run soon
#[derive(getset::Getters)]
pub struct DueTask {
    pub(super) base: TaskBase,
    pub(super) next_due_at: DateTime<Utc>,
    // due tasks have a cron schedule
    pub(super) cron_schedule: cron::Schedule
}

impl DueTask {
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            // When a task starts, its next_due_at field is updated to the next time the task is due to run
            next_due_at: calculate_next_due_at(&self.base.cron_schedule, now)?,
            base: TaskBase {
                previous_status: Some(TaskStatus::Due),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }

    pub fn is_late(&self, now: DateTime<Utc>) -> bool {
        now >= self.next_due_at + self.base.start_window
    }

    pub fn mark_late(self, now: DateTime<Utc>) -> Result<LateTask, TaskError> {
        if !self.is_late(now) {
            return Err(TaskError::InvalidStateTransition {
                from: TaskStatus::Due,
                to: TaskStatus::Late,
                details: "this task is not late".to_string(),
            });
        }
        Ok(LateTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Due),
                last_status_change_at: Some(now),
                ..self.base
            },
            next_due_at: self.next_due_at,
            cron_schedule: self.cron_schedule,
        })
    }
}

impl TryFrom<DueTask> for BoundaryTask {
    type Error = TaskError;

    fn try_from(task: DueTask) -> Result<Self, Self::Error> {
        Ok(BoundaryTask {
            status: TaskStatus::Due,
            next_due_at: calculate_next_due_at(&task.base.cron_schedule, Utc::now())?,
            ..BoundaryTask::from(task.base)
        })
    }
}

impl TryFrom<BoundaryTask> for DueTask {
    type Error = TaskError;

    fn try_from(boundary: BoundaryTask) -> Result<Self, Self::Error> {
        if boundary.status != TaskStatus::Due {
            return Err(TaskError::FailedToBuildFromBoundary {
                details: "task status must be due".to_string(),
            });
        }

        let next_due_at = boundary  
            .next_due_at
            .ok_or(TaskError::FailedToBuildFromBoundary {
                details: "Next due at is required for due task".to_string(),
            })?;
        let base: TaskBase = boundary.try_into()?;
        let cron_schedule = base
            .cron_schedule
            .clone()
            .ok_or(TaskError::FailedToBuildFromBoundary {
                details: "Cron schedule is required for due task".to_string(),
            })?;
        Ok(DueTask {
            next_due_at,
            cron_schedule,
            base,
        })
    }
}


