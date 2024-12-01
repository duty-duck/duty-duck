use chrono::{DateTime, Utc};
use super::*;

/// These are the only states a task run can be in for the related task to be healthy
pub enum HealthyTaskRun {
    Aborted(AbortedTaskRun),
    Finished(FinishedTaskRun),
}

impl From<HealthyTaskRun> for BoundaryTaskRun {
    fn from(healthy: HealthyTaskRun) -> Self {
        match healthy {
            HealthyTaskRun::Aborted(a) => a.into(),
            HealthyTaskRun::Finished(f) => f.into(),
        }
    }
}

pub struct HealthyTaskAggregate {
    pub(super) task: HealthyTask,
    pub(super) last_task_run: Option<HealthyTaskRun>,
}

impl HealthyTaskAggregate {
    /// State transition: Healthy -> Running
    /// Returns the new running task aggregate and the task run that was in the healthy state
    pub fn start(self, now: DateTime<Utc>) -> Result<(RunningTaskAggregate, Option<HealthyTaskRun>), TaskAggregateError> {
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun::new(
            *task.base().organization_id(),
            task.base().id().clone(),
            now,
        );
        Ok((RunningTaskAggregate { task, task_run }, self.last_task_run))
    }
}

