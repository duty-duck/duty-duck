use super::*;
use chrono::{DateTime, Utc};

pub struct AbsentTaskAggregate {
    pub(super) task: AbsentTask,
}

impl AbsentTaskAggregate {
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTaskAggregate, TaskAggregateError> {
        let task = self.task.start(now)?;

        let task_run = RunningTaskRun::new(task.base(), now);
        Ok(RunningTaskAggregate { task, task_run })
    }

    /// State transition to Archived
    pub fn archive(self, now: DateTime<Utc>) -> ArchivedTaskAggregate {
        ArchivedTaskAggregate {
            task: self.task.archive(now),
        }
    }
}
