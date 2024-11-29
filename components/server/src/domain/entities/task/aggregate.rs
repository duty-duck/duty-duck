use crate::domain::entities::{task::*, task_run::*};
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskAggregateError {
    #[error("Invalid state transition from {from:?} to {to:?}: {details}")]
    InvalidStateTransition {
        from: TaskRunStatus,
        to: TaskRunStatus,
        details: String,
    },
    #[error("{0}")]
    TaskError(#[from] TaskError),
    #[error("{0}")]
    TaskRunError(#[from] TaskRunError),
}

/// An enum encompassing all possible states of a task aggregate
pub enum TaskAggregate {
    /// A task that is pending, i.e. not yet due to run
    Pending(PendingTaskAggregate),
    /// A task that is due to run
    Due(DueTaskAggregate),
    /// A task that is late, i.e. outside the start window
    Late(LateTaskAggregate),
    /// A task that is running
    Running(RunningTaskAggregate),
    /// A task that is dead
    Dead(DeadTaskAggregate),
    /// A task that is absent, i.e. outside the lateness window
    Absent(AbsentTaskAggregate),
    /// A task that has completed successfully
    Finished(FinishedTaskAggregate),
    /// A task that has failed
    Failed(FailedTaskAggregate),
    /// A task that has been aborted by the user
    Aborted(AbortedTaskAggregate),
}

/// A task that is pending, i.e. not yet due to run
/// This task has no associated running task run
pub struct PendingTaskAggregate {
    task: PendingTask,
}

impl PendingTaskAggregate {
    /// Checks whether the task should transition to the due state
    pub fn is_due(&self) -> bool {
        self.task.is_due()
    }

    /// State transition: Pending -> Due
    pub fn mark_due(self) -> Result<DueTaskAggregate, TaskAggregateError> {
        Ok(DueTaskAggregate {
            task: self.task.mark_due()?,
        })
    }

    /// State transition: Pending -> Running
    pub fn start(self) -> Result<RunningTaskAggregate, TaskAggregateError> {
        let now = Utc::now();
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun {
            organization_id: *task.base().organization_id(),
            task_id: task.base().id().clone(),
            started_at: now,
            last_heartbeat_at: now,
        };
        Ok(RunningTaskAggregate { task, task_run })
    }
}

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
    pub fn start(self) -> Result<RunningTaskAggregate, TaskAggregateError> {
        let now = Utc::now();
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun {
            organization_id: *task.base().organization_id(),
            task_id: task.base().id().clone(),
            started_at: now,
            last_heartbeat_at: now,
        };
        Ok(RunningTaskAggregate { task, task_run })
    }
}

/// A task that is currently running
/// This task has an associated running task run
pub struct RunningTaskAggregate {
    task: RunningTask,
    task_run: RunningTaskRun,
}

impl RunningTaskAggregate {
    /// Checks whether the task run should transition to the dead state
    pub fn is_dead(&self) -> bool {
        Utc::now() >= self.task_run.last_heartbeat_at + self.task.heartbeat_timeout()
    }

    pub fn receive_heartbeat(self) -> Result<RunningTaskAggregate, TaskAggregateError> {
        Ok(RunningTaskAggregate {
            task_run: self.task_run.receive_heartbeat()?,
            ..self
        })
    }

    /// State transition: Running -> Dead
    pub fn mark_dead(self) -> Result<DeadTaskAggregate, TaskAggregateError> {
        if !self.is_dead() {
            return Err(TaskAggregateError::InvalidStateTransition {
                from: TaskRunStatus::Running,
                to: TaskRunStatus::Dead,
                details: "task is not dead".to_string(),
            });
        }
        Ok(DeadTaskAggregate {
            dead_task: self.task.mark_dead()?,
            dead_task_run: self.task_run.mark_dead()?,
        })
    }
}

pub struct DeadTaskAggregate {
    dead_task: DeadTask,
    dead_task_run: DeadTaskRun,
}

pub struct LateTaskAggregate {
    task: LateTask,
}

impl LateTaskAggregate {
    /// State transition: Late -> Running
    pub fn start(self) -> Result<RunningTaskAggregate, TaskAggregateError> {
        let now = Utc::now();
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun {
            organization_id: *task.base().organization_id(),
            task_id: task.base().id().clone(),
            started_at: now,
            last_heartbeat_at: now,
        };
        Ok(RunningTaskAggregate { task, task_run })
    }

    /// Checks whether the task should transition to the absent state
    /// (outside the lateness window)
    pub fn is_absent(&self) -> bool {
        self.task.is_absent()
    }

    /// State transition: Late -> Absent
    pub fn mark_absent(self) -> Result<AbsentTaskAggregate, TaskAggregateError> {
        Ok(AbsentTaskAggregate {
            task: self.task.mark_absent()?,
        })
    }
}

pub struct AbsentTaskAggregate {
    task: AbsentTask,
}

pub struct FinishedTaskAggregate {
    task: FinishedTask,
    task_run: FinishedTaskRun,
}

pub struct FailedTaskAggregate {
    task: FailedTask,
    task_run: FailedTaskRun,
}

pub struct AbortedTaskAggregate {
    task: AbortedTask,
    task_run: AbortedTaskRun,
}
