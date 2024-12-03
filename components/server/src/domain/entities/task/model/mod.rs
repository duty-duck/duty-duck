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

use super::{BoundaryTask, TaskId, TaskStatus};

mod absent;
mod due;
mod failing;
mod healthy;
mod late;
mod running;

pub use absent::*;
pub use due::*;
pub use failing::*;
pub use healthy::*;
pub use late::*;
pub use running::*;

/// Base struct with common fields shared by all task states
#[derive(getset::Getters, Debug, Clone)]
#[getset(get = "pub")]
pub struct TaskBase {
    pub(super) id: TaskId,
    pub(super) organization_id: Uuid,
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) cron_schedule: Option<croner::Cron>,
    pub(super) start_window: Duration,
    pub(super) lateness_window: Duration,
    pub(super) heartbeat_timeout: Duration,
    pub(super) created_at: DateTime<Utc>,
    pub(super) previous_status: Option<TaskStatus>,
    pub(super) last_status_change_at: Option<DateTime<Utc>>,
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
    #[error("Failed to build task from boundary: {details}")]
    FailedToBuildFromBoundary { details: String },
}

fn calculate_next_due_at(
    cron_schedule: &Option<croner::Cron>,
    now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>, TaskError> {
    if let Some(schedule) = cron_schedule {
        let next_due_at = schedule
            .find_next_occurrence(&now, false)
            .map_err(|_| TaskError::InvalidCronSchedule)?;
        Ok(Some(next_due_at))
    } else {
        Ok(None)
    }

}

impl TryFrom<BoundaryTask> for TaskBase {
    type Error = TaskError;

    fn try_from(boundary: BoundaryTask) -> Result<Self, Self::Error> {
        let cron_schedule = boundary
            .cron_schedule
            .map(|s| croner::Cron::new(&s).parse())
            .transpose()
            .map_err(|_| TaskError::InvalidCronSchedule)?;

        Ok(TaskBase {
            id: boundary.id,
            organization_id: boundary.organization_id,
            name: boundary.name,
            description: boundary.description,
            cron_schedule,
            start_window: Duration::from_secs(boundary.start_window_seconds as u64),
            lateness_window: Duration::from_secs(boundary.lateness_window_seconds as u64),
            heartbeat_timeout: Duration::from_secs(boundary.heartbeat_timeout_seconds as u64),
            created_at: boundary.created_at,
            previous_status: boundary.previous_status,
            last_status_change_at: boundary.last_status_change_at,
        })
    }
}

impl From<TaskBase> for BoundaryTask {
    fn from(base: TaskBase) -> Self {
        BoundaryTask {
            id: base.id,
            organization_id: base.organization_id,
            name: base.name,
            description: base.description,
            status: TaskStatus::Healthy, // Overridden by specific implementations
            previous_status: base.previous_status,
            last_status_change_at: base.last_status_change_at,
            cron_schedule: base.cron_schedule.map(|c| c.to_string()),
            next_due_at: None, // Overridden by specific implementations
            start_window_seconds: base.start_window.as_secs() as i32,
            lateness_window_seconds: base.lateness_window.as_secs() as i32,
            heartbeat_timeout_seconds: base.heartbeat_timeout.as_secs() as i32,
            created_at: base.created_at,
        }
    }
}