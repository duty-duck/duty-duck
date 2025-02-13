use super::*;
use chrono::{DateTime, Utc};

#[derive(getset::Getters)]
#[getset(get = "pub")]
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

    /// State transition to Archived
    pub fn archive(self, now: DateTime<Utc>) -> ArchivedTaskAggregate {
        ArchivedTaskAggregate {
            task: self.task.archive(now),
        }
    }
}
