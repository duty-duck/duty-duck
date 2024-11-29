use super::*;

/// A task that whose scheduled time has come and is expected to run soon
#[derive(getset::Getters)]
pub struct DueTask {
    pub(super) base: TaskBase,
    pub(super) next_due_at: DateTime<Utc>,
}

impl DueTask {
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Due),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }

    pub fn is_late(&self) -> bool {
        Utc::now() >= self.next_due_at + self.base.start_window
    }

    pub fn mark_late(self) -> Result<LateTask, TaskError> {
        if !self.is_late() {
            return Err(TaskError::InvalidStateTransition {
                from: TaskStatus::Due,
                to: TaskStatus::Late,
                details: "this task is not late".to_string(),
            });
        }
        Ok(LateTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Due),
                last_status_change_at: Some(Utc::now()),
                ..self.base
            },
            next_due_at: self.next_due_at,
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
        Ok(DueTask {
            next_due_at: boundary
                .next_due_at
                .ok_or(TaskError::FailedToBuildFromBoundary {
                    details: "Next due at is required for due task".to_string(),
                })?,
            base: boundary.try_into()?,
        })
    }
}


