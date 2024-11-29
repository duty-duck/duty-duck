use chrono::{DateTime, Utc};
use super::{FailingTaskAggregate, FailingTaskRun, RunningTask, RunningTaskRun, TaskAggregateError, TaskRunStatus, TaskStatus};

/// A task that is currently running
/// This task has an associated running task run
pub struct RunningTaskAggregate {
    pub(super) task: RunningTask,
    pub(super) task_run: RunningTaskRun,
}

impl RunningTaskAggregate {
    pub fn receive_heartbeat(self, now: DateTime<Utc>) -> Result<RunningTaskAggregate, TaskAggregateError> {
        Ok(RunningTaskAggregate {
            task_run: self.task_run.receive_heartbeat(now)?,
            ..self
        })
    }

    pub fn is_dead(&self, now: DateTime<Utc>) -> bool {
        now >= *self.task_run.last_heartbeat_at() + self.task.heartbeat_timeout()
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
