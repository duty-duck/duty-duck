mod absent;
mod due;
mod failing;
mod healthy;
mod late;
mod pending;
mod running;

pub use absent::*;
pub use due::*;
pub use failing::*;
pub use healthy::*;
pub use late::*;
pub use pending::*;
pub use running::*;

use crate::domain::{
    entities::{task::*, task_run::*},
    ports::{
        task_repository::TaskRepository,
        task_run_repository::{ListTaskRunsOpts, TaskRunRepository},
    },
};
use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TaskAggregateError {
    #[error("Inconsistent task run state: {task_id:?} {task_run_status:?} {details}")]
    InconsistentTaskRunState {
        task_id: TaskId,
        task_run_status: TaskRunStatus,
        details: String,
    },
    #[error("{0}")]
    TaskError(#[from] TaskError),
    #[error("{0}")]
    TaskRunError(#[from] TaskRunError),
    #[error("Invalid state transition: {from:?} -> {to:?} {details}")]
    InvalidStateTransition {
        from: (TaskStatus, Option<TaskRunStatus>),
        to: (TaskStatus, Option<TaskRunStatus>),
        details: String,
    },
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
    /// A task that is failing
    Failing(FailingTaskAggregate),
    /// A task that is healthy
    Healthy(HealthyTaskAggregate),
    /// A task that is absent, i.e. scheduled to run but did not start within the lateness window
    Absent(AbsentTaskAggregate),
}

/// Retrive a task aggregate from the database by its id
pub async fn get_task_aggregate<TR, TRR>(
    task_repository: &TR,
    task_run_repository: &TRR,
    tx: &mut TR::Transaction,
    organization_id: Uuid,
    task_id: &TaskId,
) -> anyhow::Result<Option<TaskAggregate>>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
{
    match task_repository
        .get_task(tx, organization_id, task_id)
        .await?
    {
        Some(task) => {
            let aggregate: TaskAggregate = match task.status {
                TaskStatus::Running => {
                    let task_run = task_run_repository
                        .get_latest_task_run(
                            tx,
                            organization_id,
                            task_id,
                            &[TaskRunStatus::Running],
                        )
                        .await?
                        .ok_or(TaskAggregateError::InconsistentTaskRunState {
                            task_id: task_id.clone(),
                            task_run_status: TaskRunStatus::Running,
                            details: "no running task run found".to_string(),
                        })?;

                    TaskAggregate::Running(RunningTaskAggregate {
                        task: task.try_into()?,
                        task_run: task_run.try_into()?,
                    })
                }
                TaskStatus::Failing => {
                    let task_run = task_run_repository
                        .get_latest_task_run(
                            tx,
                            organization_id,
                            task_id,
                            &[TaskRunStatus::Failed, TaskRunStatus::Dead],
                        )
                        .await?
                        .ok_or(TaskAggregateError::InconsistentTaskRunState {
                            task_id: task_id.clone(),
                            task_run_status: TaskRunStatus::Failed,
                            details: "no failed or dead task run found".to_string(),
                        })?;

                    match task_run.status {
                        TaskRunStatus::Failed => TaskAggregate::Failing(FailingTaskAggregate {
                            task: task.try_into()?,
                            task_run: FailingTaskRun::Failed(task_run.try_into()?),
                        }),
                        TaskRunStatus::Dead => TaskAggregate::Failing(FailingTaskAggregate {
                            task: task.try_into()?,
                            task_run: FailingTaskRun::Dead(task_run.try_into()?),
                        }),
                        _ => unreachable!(),
                    }
                }
                TaskStatus::Pending => TaskAggregate::Pending(PendingTaskAggregate {
                    task: task.try_into()?,
                }),
                TaskStatus::Due => TaskAggregate::Due(DueTaskAggregate {
                    task: task.try_into()?,
                }),
                TaskStatus::Late => TaskAggregate::Late(LateTaskAggregate {
                    task: task.try_into()?,
                }),
                TaskStatus::Absent => TaskAggregate::Absent(AbsentTaskAggregate {
                    task: task.try_into()?,
                }),
                TaskStatus::Healthy => {
                    let last_task_run = task_run_repository
                        .get_latest_task_run(
                            tx,
                            organization_id,
                            task_id,
                            &[TaskRunStatus::Aborted, TaskRunStatus::Finished],
                        )
                        .await?;

                    let last_task_run = match last_task_run {
                        Some(task_run) => Some(match task_run.status {
                            TaskRunStatus::Aborted => HealthyTaskRun::Aborted(task_run.try_into()?),
                            TaskRunStatus::Finished => {
                                HealthyTaskRun::Finished(task_run.try_into()?)
                            }
                            _ => unreachable!(),
                        }),
                        None => None,
                    };

                    TaskAggregate::Healthy(HealthyTaskAggregate {
                        task: task.try_into()?,
                        last_task_run,
                    })
                }
            };

            Ok(Some(aggregate))
        }
        None => Ok(None),
    }
}
