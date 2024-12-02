
use super::*;

/// A task that was running, and finished unsuccessfully
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
            next_due_at: calculate_next_due_at(&self.base.cron_schedule, now)?,
            base: TaskBase {
                previous_status: Some(TaskStatus::Failing),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
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
