use super::TaskRunStatus;
/// This module defines the different states a task run can be in using separate structs
/// Together, these structs form a finite state machine that encompasses all possible states and all legal transitions between them.
///
/// To persist this state machine or serve it over the network, we use the `BoundaryTaskRun` type.
use thiserror::Error;

mod aborted;
mod dead;
mod failed;
mod finished;
mod running;

pub use aborted::AbortedTaskRun;
pub use dead::DeadTaskRun;
pub use failed::FailedTaskRun;
pub use finished::FinishedTaskRun;
pub use running::RunningTaskRun;

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
