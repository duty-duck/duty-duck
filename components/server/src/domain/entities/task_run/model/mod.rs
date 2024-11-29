///! This module defines the different states a task run can be in using separate structs
/// Together, these structs form a finite state machine that encompasses all possible states and all legal transitions between them.
///
/// To persist this state machine or serve it over the network, we use the `BoundaryTaskRun` type.
use thiserror::Error;
use super::TaskRunStatus;

mod running;
mod finished;
mod failed;
mod aborted;
mod dead;

pub use running::RunningTaskRun;
pub use finished::FinishedTaskRun;
pub use failed::FailedTaskRun;
pub use aborted::AbortedTaskRun;
pub use dead::DeadTaskRun;

#[derive(Debug, Error)]
pub enum TaskRunError {
    #[error("Invalid state transition from {from:?} to {to:?}: {details}")]
    InvalidStateTransition {
        from: TaskRunStatus,
        to: TaskRunStatus,
        details: String,
    },
    #[error("Failed to build task run from boundary: {details}")]
    FailedToBuildFromBoundary { details: String },
}