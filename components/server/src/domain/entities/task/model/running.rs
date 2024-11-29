use super::*;
use chrono::Utc;
use std::time::Duration;

/// A task that is currently running
#[derive(getset::Getters)]
pub struct RunningTask {
    #[getset(get = "pub")]
    pub(super) base: TaskBase,
}

impl RunningTask {
    pub fn finish(self, now: DateTime<Utc>) -> Result<HealthyTask, TaskError> {
        Ok(HealthyTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Running),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }

    pub fn fail(self, now: DateTime<Utc>) -> Result<FailingTask, TaskError> {
        Ok(FailingTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Running),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }

    pub fn heartbeat_timeout(&self) -> Duration {
        self.base.heartbeat_timeout
    }
}

impl TryFrom<RunningTask> for BoundaryTask {
    type Error = TaskError;

    fn try_from(task: RunningTask) -> Result<Self, Self::Error> {
        Ok(BoundaryTask {
            status: TaskStatus::Running,
            next_due_at: calculate_next_due_at(&task.base.cron_schedule, Utc::now())?,
            ..BoundaryTask::from(task.base)
        })
    }
}

impl TryFrom<BoundaryTask> for RunningTask {
    type Error = TaskError;

    fn try_from(boundary: BoundaryTask) -> Result<Self, Self::Error> {
        if boundary.status != TaskStatus::Running {
            return Err(TaskError::FailedToBuildFromBoundary {
                details: "task status must be running".to_string(),
            });
        }
        Ok(RunningTask {
            base: boundary.try_into()?,
        })
    }
}

