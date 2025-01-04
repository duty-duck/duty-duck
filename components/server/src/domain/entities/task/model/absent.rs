use super::*;

/// A task that was scheduled to run, ran late, and eventually did not run at all
pub struct AbsentTask {
    pub(super) base: TaskBase,
    pub(super) next_due_at: DateTime<Utc>,
    // absent tasks have a cron schedule
    pub(super) cron_schedule: cron::Schedule,
}

impl AbsentTask {
    /// State transition: Absent -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            next_due_at: calculate_next_due_at(&self.base.cron_schedule, now)?,
            base: self.base,
        })
    }

    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        now >= self.next_due_at
    }

    /// State transition: Absent -> Due
    pub fn mark_due(self, now: DateTime<Utc>) -> Result<DueTask, TaskError> {
        if !self.is_due(now) {
            return Err(TaskError::InvalidStateTransition {
                from: TaskStatus::Absent,
                to: TaskStatus::Due,
                details: "this task has a cron schedule but is not due to run yet".to_string(),
            });
        }
        Ok(DueTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Absent),
                last_status_change_at: Some(now),
                ..self.base
            },
            cron_schedule: self.cron_schedule,
            next_due_at: self.next_due_at,
        })
    }
}

impl TryFrom<AbsentTask> for BoundaryTask {
    type Error = TaskError;

    fn try_from(task: AbsentTask) -> Result<Self, Self::Error> {
        Ok(BoundaryTask {
            status: TaskStatus::Absent,
            next_due_at: calculate_next_due_at(&task.base.cron_schedule, Utc::now())?,
            ..BoundaryTask::from(task.base)
        })
    }
}

impl TryFrom<BoundaryTask> for AbsentTask {
    type Error = TaskError;

    fn try_from(boundary: BoundaryTask) -> Result<Self, Self::Error> {
        if boundary.status != TaskStatus::Absent {
            return Err(TaskError::FailedToBuildFromBoundary {
                details: "task status must be absent".to_string(),
            });
        }
        let next_due_at = boundary
            .next_due_at
            .ok_or(TaskError::FailedToBuildFromBoundary {
                details: "Next due at is required for absent task".to_string(),
            })?;

        let base: TaskBase = boundary.try_into()?;
        let cron_schedule = base
            .cron_schedule
            .clone()
            .ok_or(TaskError::FailedToBuildFromBoundary {
                details: "Cron schedule is required for absent task".to_string(),
            })?;

        Ok(AbsentTask {
            next_due_at,
            cron_schedule,
            base,
        })
    }
}
