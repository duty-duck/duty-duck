use chrono::{DateTime, Utc};

use super::*;

/// These are the only states a task run can be in for the related task to be failing
#[derive(Debug)]
pub enum FailingTaskRun {
    Failed(FailedTaskRun),
    Dead(DeadTaskRun),
}

impl From<FailingTaskRun> for BoundaryTaskRun {
    fn from(failing: FailingTaskRun) -> Self {
        match failing {
            FailingTaskRun::Failed(f) => f.into(),
            FailingTaskRun::Dead(d) => d.into(),
        }
    }
}

#[derive(getset::Getters, Debug)]
#[getset(get = "pub")]
pub struct FailingTaskAggregate {
    pub(super) task: FailingTask,
    pub(super) task_run: FailingTaskRun,
}

impl FailingTaskAggregate {
    /// State transition: Failing -> Running
    /// Returns the new running task aggregate and the task run that was in the failing state
    pub fn start(
        self,
        now: DateTime<Utc>,
    ) -> Result<(RunningTaskAggregate, FailingTaskRun), TaskAggregateError> {
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun::new(task.base(), now);
        Ok((RunningTaskAggregate { task, task_run }, self.task_run))
    }

    /// State transition: Failing -> Due
    pub fn mark_due(self, now: DateTime<Utc>) -> Result<DueTaskAggregate, TaskAggregateError> {
        let task = self.task.mark_due(now)?;
        Ok(DueTaskAggregate { task })
    }

    /// State transition to Archived
    pub fn archive(self, now: DateTime<Utc>) -> ArchivedTaskAggregate {
        ArchivedTaskAggregate {
            task: self.task.archive(now),
        }
    }
}
