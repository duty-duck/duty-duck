use super::*;

/// A task that was scheduled to run, ran late, and eventually did not run at all
pub struct AbsentTask {
    pub(super) base: TaskBase,
    #[allow(unused)]
    // in the context of an absent task, the next_due_at field is the last time the task was due to run before it was marked absent,
    // i.e. the time of the expected run that never happened
    pub(super) next_due_at: DateTime<Utc>,
    // absent tasks have a cron schedule
    #[allow(unused)]
    pub(super) cron_schedule: cron::Schedule,
}

impl AbsentTask {
    /// State transition: Absent -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            next_due_at: calculate_next_due_at(
                self.base.cron_schedule.as_ref(),
                self.base.schedule_timezone.as_ref(),
                now,
            )?,
            base: self.base,
        })
    }
}

impl TryFrom<AbsentTask> for BoundaryTask {
    type Error = TaskError;

    fn try_from(task: AbsentTask) -> Result<Self, Self::Error> {
        Ok(BoundaryTask {
            status: TaskStatus::Absent,
            // Next due at is required for absent tasks, and it's not present in the base task,
            // so we need to add it here
            next_due_at: Some(task.next_due_at),
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
        let cron_schedule =
            base.cron_schedule
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
