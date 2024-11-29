use super::*;
use chrono::{DateTime, Utc};

/// A task that is pending, i.e. not yet due to run
/// This task has no associated running task run
pub struct PendingTaskAggregate {
    pub(super) task: PendingTask,
}

impl PendingTaskAggregate {
    /// Checks whether the task should transition to the due state
    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        self.task.is_due(now)
    }

    /// State transition: Pending -> Due
    pub fn mark_due(self, now: DateTime<Utc>) -> Result<DueTaskAggregate, TaskAggregateError> {
        Ok(DueTaskAggregate {
            task: self.task.mark_due(now)?,
        })
    }

    /// State transition: Pending -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTaskAggregate, TaskAggregateError> {
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun::new(
            *task.base().organization_id(),
            task.base().id().clone(),
            now,
        );
        Ok(RunningTaskAggregate { task, task_run })
    }
}