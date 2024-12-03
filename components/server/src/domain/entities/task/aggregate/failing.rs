use chrono::{DateTime, Utc};

use super::*;

/// These are the only states a task run can be in for the related task to be failing
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

pub struct FailingTaskAggregate {
    pub(super) task: FailingTask,
    pub(super) task_run: FailingTaskRun,
}


impl FailingTaskAggregate {
    /// State transition: Failing -> Running
    /// Returns the new running task aggregate and the task run that was in the failing state
    pub fn start(self, now: DateTime<Utc>) -> Result<(RunningTaskAggregate, FailingTaskRun), TaskAggregateError> {
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun::new(
            *task.base().organization_id(),
            task.base().id().clone(),
            now,
            *task.base().heartbeat_timeout(),
        );
        Ok((RunningTaskAggregate { task, task_run }, self.task_run))
    }

    /// State transition: Failing -> Due
    pub fn mark_due(self, now: DateTime<Utc>) -> Result<DueTaskAggregate, TaskAggregateError> {
        let task = self.task.mark_due(now)?;
        Ok(DueTaskAggregate { task })
    }
}
