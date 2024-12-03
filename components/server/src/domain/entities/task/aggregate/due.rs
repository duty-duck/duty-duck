use super::{DueTask, LateTaskAggregate, RunningTaskAggregate, RunningTaskRun, TaskAggregateError};
use chrono::{DateTime, Utc};

pub struct DueTaskAggregate {
    pub task: DueTask,
}

impl DueTaskAggregate {
    /// Checks whether the task should transition to the absent state
    pub fn is_late(&self) -> bool {
        self.task.is_late()
    }

    /// State transition: Due -> Late
    pub fn mark_late(self) -> Result<LateTaskAggregate, TaskAggregateError> {
        Ok(LateTaskAggregate {
            task: self.task.mark_late()?,
        })
    }

    /// State transition: Due -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTaskAggregate, TaskAggregateError> {
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun::new(
            *task.base().organization_id(),
            task.base().id().clone(),
            now,
            *task.base().heartbeat_timeout(),
        );
        Ok(RunningTaskAggregate { task, task_run })
    }
}