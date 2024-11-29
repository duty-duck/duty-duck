//! This module defines the different states a task can be in using separate structs
//! Together, these structs form a finite state machine that encompasses all possible states and all legal transitions between them.
//!
//! A task is always in one of these states, and can only transition to a valid subsequent state.
//! The `TaskBase` struct contains the fields that are common to all states, and the individual state structs
//! contain the fields that are specific to that state.
//!
//! State transitions are managed through method calls on the state structs, and return a new state struct of the target state.
//! If the transition is invalid, the method returns an error.
//!
//! A task can be scheduled to run at a given cron schedule or not. Scheduled tasks have a start window and a lateness window,
//! which together determine when a task transitions from `Due` to `Late` and from `Late` to `Absent`.
//!
//! Here's a visual representation of a task's timeline:
//!
//! ```
//! -------------- next due at --------- next due at + start window ------------ next_due_at + start window + lateness window --------->
//! --------------------|----------------------------|------------------------------------------------|-------------------------------->
//! ---------X----------|--------------X-------------|---------------------X--------------------------|---------X---------------------->
//! ----- pending ------|----------- due ------------|------------------ late ------------------------|------ absent ------------------>
//! ```
use std::time::Duration;

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use super::{TaskId, TaskStatus};

/// Base struct with common fields shared by all task states
#[derive(getset::Getters)]
#[getset(get = "pub")]
pub struct TaskBase {
    id: TaskId,
    organization_id: Uuid,
    name: String,
    description: Option<String>,
    cron_schedule: Option<croner::Cron>,
    start_window: Duration,
    lateness_window: Duration,
    heartbeat_timeout: Duration,
    created_at: DateTime<Utc>,
    previous_status: Option<TaskStatus>,
    last_status_change_at: Option<DateTime<Utc>>,
}

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Invalid cron schedule")]
    InvalidCronSchedule,
    #[error("Invalid state transition from {from:?} to {to:?}: {details}")]
    InvalidStateTransition {
        from: TaskStatus,
        to: TaskStatus,
        details: String,
    },
}

// State-specific structs

/// A task that is not scheduled to run
#[derive(getset::Getters)]
pub struct InactiveTask {
    base: TaskBase,
}

impl InactiveTask {
    /// State transition: Inactive -> Pending
    pub fn activate(self) -> Result<PendingTask, TaskError> {
        let now = Utc::now();
        Ok(PendingTask {
            next_due_at: calculate_next_due_at(&self.base.cron_schedule, now)?,
            base: TaskBase {
                previous_status: Some(TaskStatus::Inactive),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }
}

/// A task that is scheduled to run at a given time in the future
#[derive(getset::Getters)]
pub struct PendingTask {
    base: TaskBase,
    next_due_at: DateTime<Utc>,
}

impl PendingTask {
    pub fn is_due(&self) -> bool {
        Utc::now() >= self.next_due_at
    }

    /// State transition: Pending -> Due
    pub fn mark_due(self) -> Result<DueTask, TaskError> {
        if !self.is_due() {
            return Err(TaskError::InvalidStateTransition {
                from: TaskStatus::Pending,
                to: TaskStatus::Due,
                details: "this task is not due to run yet".to_string(),
            });
        }
        Ok(DueTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Pending),
                last_status_change_at: Some(Utc::now()),
                ..self.base
            },
            next_due_at: self.next_due_at,
        })
    }

    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Pending),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }
}

/// A task that whose scheduled time has come and is expected to run soon
#[derive(getset::Getters)]
pub struct DueTask {
    base: TaskBase,
    next_due_at: DateTime<Utc>,
}

impl DueTask {
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Due),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }

    pub fn is_late(&self) -> bool {
        Utc::now() >= self.next_due_at + self.base.start_window
    }

    pub fn mark_late(self) -> Result<LateTask, TaskError> {
        if !self.is_late() {
            return Err(TaskError::InvalidStateTransition {
                from: TaskStatus::Due,
                to: TaskStatus::Late,
                details: "this task is not late".to_string(),
            });
        }
        Ok(LateTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Due),
                last_status_change_at: Some(Utc::now()),
                ..self.base
            },
            next_due_at: self.next_due_at,
        })
    }
}

/// A task that was scheduled to run, but ran late.
/// The task has not yet started running, but is still inside the lateness window
#[derive(getset::Getters)]
pub struct LateTask {
    base: TaskBase,
    next_due_at: DateTime<Utc>,
}

impl LateTask {
    /// Checks whether the task should transition to the absent state
    /// (outside the lateness window)
    pub fn is_absent(&self) -> bool {
        Utc::now() >= self.next_due_at + self.base.lateness_window
    }

    /// State transition: Late -> Absent
    pub fn mark_absent(self) -> Result<AbsentTask, TaskError> {
        if !self.is_absent() {
            return Err(TaskError::InvalidStateTransition {
                from: TaskStatus::Late,
                to: TaskStatus::Absent,
                details: "this task is not absent".to_string(),
            });
        }
        Ok(AbsentTask {
            next_due_at: calculate_next_due_at(&self.base.cron_schedule, Utc::now())?,
            base: TaskBase {
                previous_status: Some(TaskStatus::Late),
                last_status_change_at: Some(Utc::now()),
                ..self.base
            },
        })
    }

    /// State transition: Late -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Due),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }
}

/// A task that is currently running
#[derive(getset::Getters)]
pub struct RunningTask {
    #[getset(get = "pub")]
    base: TaskBase,
}

impl RunningTask {
    pub fn finish(self) -> Result<FinishedTask, TaskError> {
        Ok(FinishedTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Running),
                last_status_change_at: Some(Utc::now()),
                ..self.base
            },
        })
    }

    pub fn fail(self) -> Result<FailedTask, TaskError> {
        Ok(FailedTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Running),
                last_status_change_at: Some(Utc::now()),
                ..self.base
            },
        })
    }

    pub fn abort(self) -> Result<AbortedTask, TaskError> {
        Ok(AbortedTask {
            base: TaskBase {
                previous_status: Some(TaskStatus::Running),
                last_status_change_at: Some(Utc::now()),
                ..self.base
            },
        })
    }

    pub fn mark_dead(self) -> Result<DeadTask, TaskError> {
        Ok(DeadTask {
            next_due_at: calculate_next_due_at(&self.base.cron_schedule, Utc::now())?,
            base: TaskBase {
                previous_status: Some(TaskStatus::Running),
                last_status_change_at: Some(Utc::now()),
                ..self.base
            },
        })
    }

    pub fn heartbeat_timeout(&self) -> Duration {
        self.base.heartbeat_timeout
    }
}

/// A task that was scheduled to run, ran late, and eventually did not run at all
/// This is a terminal state
pub struct AbsentTask {
    base: TaskBase,
    next_due_at: DateTime<Utc>,
}

/// A task that was running, but did not send a heartbeat in a while
/// This is a terminal state
pub struct DeadTask {
    base: TaskBase,
    next_due_at: DateTime<Utc>,
}

/// A task that was running, but was aborted by a user
/// This is a terminal state
pub struct AbortedTask {
    base: TaskBase,
}

/// A task that was running, and finished successfully
/// This is a terminal state
pub struct FinishedTask {
    base: TaskBase,
}

/// A task that was running, and finished unsuccessfully
/// This is a terminal state
pub struct FailedTask {
    base: TaskBase,
}

fn calculate_next_due_at(
    cron_schedule: &Option<croner::Cron>,
    now: DateTime<Utc>,
) -> Result<DateTime<Utc>, TaskError> {
    if let Some(schedule) = cron_schedule {
        let next_due_at = schedule
            .find_next_occurrence(&now, true)
            .map_err(|_| TaskError::InvalidCronSchedule)?;
        Ok(next_due_at)
    } else {
        Ok(now)
    }
}
