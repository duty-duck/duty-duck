use super::*;

/// A task that is in a healthy state (not failed, not late, not failing)
pub struct HealthyTask {
    pub(super) base: TaskBase,
}


impl TryFrom<HealthyTask> for BoundaryTask {
    type Error = TaskError;

    fn try_from(task: HealthyTask) -> Result<Self, Self::Error> {
        Ok(BoundaryTask {
            status: TaskStatus::Absent,
            next_due_at: calculate_next_due_at(&task.base.cron_schedule, Utc::now())?,
            ..BoundaryTask::from(task.base)
        })
    }
}

impl TryFrom<BoundaryTask> for HealthyTask {
    type Error = TaskError;

    fn try_from(boundary: BoundaryTask) -> Result<Self, Self::Error> {
        if boundary.status != TaskStatus::Healthy {
            return Err(TaskError::FailedToBuildFromBoundary {
                details: "task status must be healthy".to_string(),
            });
        }
        Ok(HealthyTask {
            base: boundary.try_into()?,
        })
    }
}
