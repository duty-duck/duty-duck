use super::{DueTask, LateTaskAggregate, RunningTaskAggregate, RunningTaskRun, TaskAggregateError};
use chrono::{DateTime, Utc};

pub struct DueTaskAggregate {
    pub task: DueTask,
}

impl DueTaskAggregate {
    /// State transition: Due -> Late
    pub fn mark_late(self, now: DateTime<Utc>) -> Result<LateTaskAggregate, TaskAggregateError> {
        Ok(LateTaskAggregate {
            task: self.task.mark_late(now)?,
        })
    }

    /// State transition: Due -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTaskAggregate, TaskAggregateError> {
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun::new(task.base(), now);
        Ok(RunningTaskAggregate { task, task_run })
    }
}
