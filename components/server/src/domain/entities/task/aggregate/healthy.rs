use crate::domain::use_cases::tasks::CreateTaskCommand;
use chrono::{DateTime, Utc};

use super::*;

/// These are the only states a task run can be in for the related task to be healthy
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct HealthyTaskAggregate {
    pub(super) task: HealthyTask,
    pub(super) last_task_run: Option<HealthyTaskRun>,
}

impl HealthyTaskAggregate {
    pub fn new(
        organization_id: Uuid,
        command: CreateTaskCommand,
    ) -> Result<Self, TaskAggregateError> {
        let task = HealthyTask::new(organization_id, command)?;
        Ok(Self {
            task,
            last_task_run: None,
        })
    }

    /// State transition: Healthy -> Running
    /// Returns the new running task aggregate and the task run that was in the healthy state
    pub fn start(
        self,
        now: DateTime<Utc>,
    ) -> Result<(RunningTaskAggregate, Option<HealthyTaskRun>), TaskAggregateError> {
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun::new(task.base(), now);
        Ok((RunningTaskAggregate { task, task_run }, self.last_task_run))
    }

    /// State transition: Healthy -> Due
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
