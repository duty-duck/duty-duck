use super::*;

/// A task that was scheduled to run, ran late, and eventually did not run at all
pub struct AbsentTask {
    pub(super) base: TaskBase,
    pub(super) next_due_at: DateTime<Utc>,
}

impl AbsentTask {
    /// State transition: Absent -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            next_due_at: calculate_next_due_at(&self.base.cron_schedule, now)?,
            base: self.base,
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
        Ok(AbsentTask {
            next_due_at: boundary
                .next_due_at
                .ok_or(TaskError::FailedToBuildFromBoundary {
                    details: "Next due at is required for absent task".to_string(),
                })?,
            base: boundary.try_into()?,
        })
    }
}
