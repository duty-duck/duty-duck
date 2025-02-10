use crate::domain::entities::task::{BoundaryTask, TaskStatus};

use super::{TaskBase, TaskError};

/// A task that is archived (can no longer be interacted with, and whose name can repurposed for another task)
#[derive(Debug, Clone, getset::Getters)]
#[getset(get = "pub")]
pub struct ArchivedTask {
    pub(super) base: TaskBase,
}

impl TryFrom<ArchivedTask> for BoundaryTask {
    type Error = TaskError;

    fn try_from(task: ArchivedTask) -> Result<Self, Self::Error> {
        Ok(BoundaryTask {
            status: TaskStatus::Archived,
            ..BoundaryTask::from(task.base)
        })
    }
}

impl TryFrom<BoundaryTask> for ArchivedTask {
    type Error = TaskError;

    fn try_from(boundary: BoundaryTask) -> Result<Self, Self::Error> {
        if boundary.status != TaskStatus::Archived {
            return Err(TaskError::FailedToBuildFromBoundary {
                details: "task status must be archived".to_string(),
            });
        }
        Ok(ArchivedTask {
            base: boundary.try_into()?,
        })
    }
}
