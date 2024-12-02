use super::{
    FailingTaskAggregate, FailingTaskRun, HealthyTaskAggregate, HealthyTaskRun, RunningTask,
    RunningTaskRun, TaskAggregateError, TaskRunStatus, TaskStatus,
};
use chrono::{DateTime, Utc};

/// A task that is currently running
/// This task has an associated running task run
#[derive(Debug, Clone)]
pub struct RunningTaskAggregate {
    pub(super) task: RunningTask,
    pub(super) task_run: RunningTaskRun,
}

impl RunningTaskAggregate {
    pub fn receive_heartbeat(
        self,
        now: DateTime<Utc>,
    ) -> Result<RunningTaskAggregate, TaskAggregateError> {
        Ok(RunningTaskAggregate {
            task_run: self.task_run.receive_heartbeat(now)?,
            ..self
        })
    }

    pub fn is_dead(&self, now: DateTime<Utc>) -> bool {
        now >= *self.task_run.last_heartbeat_at() + self.task.heartbeat_timeout()
    }

    /// State transition: Running -> Healthy
    pub fn mark_finished(
        self,
        now: DateTime<Utc>,
        exit_code: Option<i32>,
    ) -> Result<HealthyTaskAggregate, TaskAggregateError> {
        Ok(HealthyTaskAggregate {
            task: self.task.finish(now)?,
            last_task_run: Some(HealthyTaskRun::Finished(
                self.task_run.mark_finished(now, exit_code)?,
            )),
        })
    }

    /// State transition: Running -> Failed
    pub fn mark_failed(
        self,
        now: DateTime<Utc>,
        exit_code: Option<i32>,
        error_message: Option<String>,
    ) -> Result<FailingTaskAggregate, TaskAggregateError> {
        Ok(FailingTaskAggregate {
            task: self.task.fail(now)?,
            task_run: FailingTaskRun::Failed(self.task_run.mark_failed(now, exit_code, error_message)?),
        })
    }

    /// State transition: Running -> Dead
    pub fn mark_dead(self, now: DateTime<Utc>) -> Result<FailingTaskAggregate, TaskAggregateError> {
        if !self.is_dead(now) {
            return Err(TaskAggregateError::InvalidStateTransition {
                from: (TaskStatus::Running, Some(TaskRunStatus::Running)),
                to: (TaskStatus::Failing, Some(TaskRunStatus::Dead)),
                details: "task run is not dead".to_string(),
            });
        }
        Ok(FailingTaskAggregate {
            task: self.task.fail(now)?,
            task_run: FailingTaskRun::Dead(self.task_run.mark_dead(now)?),
        })
    }
}
