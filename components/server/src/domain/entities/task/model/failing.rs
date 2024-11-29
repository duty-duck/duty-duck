use crate::domain::entities::task_run::{DeadTaskRun, FailedTaskRun};

use super::*;

/// A task that was running, and finished unsuccessfully
pub struct FailingTask {
    pub(super) base: TaskBase,
}


impl TryFrom<FailingTask> for BoundaryTask {
    type Error = TaskError;

    fn try_from(task: FailingTask) -> Result<Self, Self::Error> {
        Ok(BoundaryTask {
            status: TaskStatus::Failing,
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
            base: boundary.try_into()?,
        })
    }
}
