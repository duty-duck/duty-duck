mod absent;
mod archived;
mod due;
mod failing;
mod healthy;
mod late;
mod running;

pub use absent::*;
use anyhow::Context;
pub use archived::*;
use chrono::{DateTime, Utc};
pub use due::*;
pub use failing::*;
pub use healthy::*;
pub use late::*;
pub use running::*;

use crate::domain::{
    entities::{task::*, task_run::*},
    ports::{task_repository::TaskRepository, task_run_repository::TaskRunRepository},
    use_cases::tasks::{UpdateTaskCommand, UpdateTaskError},
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TaskAggregateError {
    #[error("Inconsistent task run state: {task_id:?} {task_run_status:?} {details}")]
    InconsistentTaskRunState {
        task_id: TaskUserId,
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
    /// An archived task, that can no longer be interacted with but can still be read
    Archived(ArchivedTaskAggregate),
}

/// Retrieve a task aggregate from the database by its id (user-defined or UUID)
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
        .get_task_by_id(tx, organization_id, task_id)
        .await?
    {
        Some(task) => {
            let aggregate: TaskAggregate = match task.status {
                TaskStatus::Running => {
                    let task_run = task_run_repository
                        .get_latest_task_run(
                            tx,
                            organization_id,
                            task.id,
                            &[TaskRunStatus::Running],
                        )
                        .await?;

                    from_boundary(task, task_run)?
                }
                TaskStatus::Failing => {
                    let task_run = task_run_repository
                        .get_latest_task_run(
                            tx,
                            organization_id,
                            task.id,
                            &[TaskRunStatus::Failed, TaskRunStatus::Dead],
                        )
                        .await?;

                    from_boundary(task, task_run)?
                }
                TaskStatus::Due => from_boundary(task, None)?,
                TaskStatus::Late => from_boundary(task, None)?,
                TaskStatus::Absent => from_boundary(task, None)?,
                TaskStatus::Healthy => {
                    let last_task_run = task_run_repository
                        .get_latest_task_run(
                            tx,
                            organization_id,
                            task.id,
                            &[TaskRunStatus::Aborted, TaskRunStatus::Finished],
                        )
                        .await?;

                    from_boundary(task, last_task_run)?
                }
                TaskStatus::Archived => from_boundary(task, None)?,
            };

            Ok(Some(aggregate))
        }
        None => Ok(None),
    }
}

/// Saves the task aggregate to the database.
/// Persist changes made to the task aggregate to the database.
pub async fn save_task_aggregate<TR, TRR>(
    task_repository: &TR,
    task_run_repository: &TRR,
    tx: &mut TR::Transaction,
    aggregate: TaskAggregate,
) -> anyhow::Result<()>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
{
    let (boundary_task, boundary_task_run) =
        to_boundary(aggregate).context("failed to convert task aggregate to boundary")?;
    task_repository
        .upsert_task(tx, boundary_task)
        .await
        .context("failed to upsert task to the database")?;
    if let Some(boundary_task_run) = boundary_task_run {
        task_run_repository
            .upsert_task_run(tx, boundary_task_run)
            .await
            .context("failed to upsert task run to the database")?;
    }

    Ok(())
}

/// Applies user modifications to the task aggregate.
/// Note that the aggregate may change its type as a result of the modifications; for instance, a late task
/// may become a healthy task as a result of the user changeing its schedule. For this reason, this function operators
/// on the [TaskAggregate] enum and returns a new [TaskAggregate], not on a specific variant.
///
/// The changes won't be fully persisted until [save_task_aggregate] is called and the transaction is committed.
#[tracing::instrument(skip(task_repository, task_run_repository, aggregate, tx))]
pub async fn update_task_aggregate<TR, TRR>(
    task_repository: &TR,
    task_run_repository: &TRR,
    tx: &mut TR::Transaction,
    aggregate: TaskAggregate,
    command: UpdateTaskCommand,
    now: DateTime<Utc>,
) -> Result<TaskAggregate, UpdateTaskError>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
{
    // Retrieve the task base
    let task_base = match &aggregate {
        TaskAggregate::Archived(_) => return Err(UpdateTaskError::TaskArchived),
        TaskAggregate::Healthy(agg) => agg.task().base(),
        TaskAggregate::Late(agg) => agg.task().base(),
        TaskAggregate::Absent(agg) => agg.task().base(),
        TaskAggregate::Failing(agg) => agg.task().base(),
        TaskAggregate::Due(agg) => agg.task().base(),
        TaskAggregate::Running(agg) => agg.task().base(),
    };

    // Check the new id (if the id changes) is available
    if let Some(new_id) = &command.user_id {
        if new_id != &task_base.user_id {
            let existing_task = task_repository
                .get_task_by_user_id(tx, task_base.organization_id, new_id)
                .await?;
            if existing_task.is_some() {
                return Err(UpdateTaskError::UserIdConflict);
            }
        }
    }

    // Update the task base with new information from the user
    let updated_task_base = task_base.update(command)?;

    // Retrieve the last task run of the task
    let last_task_run = task_run_repository
        .get_latest_task_run(tx, updated_task_base.id, updated_task_base.id, &[])
        .await?;

    // Create a new task aggregate from the updated task base. The type of the aggregate will depend on the status
    // of the last task run (if any). The type of the resulting aggregate will be "healthy", "failing" or "running".
    // The "due", "late" and "absent" states are handles by the background tasks in charge of collecting due/late/absent tasks.
    // The "archived" state is simply not possible to update.
    let agg = match last_task_run {
        None => {
            let task = HealthyTask::from_task_base(updated_task_base, now)?;
            TaskAggregate::Healthy(HealthyTaskAggregate {
                task,
                last_task_run: None,
            })
        }
        Some(r) => match r.status {
            TaskRunStatus::Running => {
                let task = RunningTask::from_task_base(updated_task_base, now)?;
                let task_run =
                    RunningTaskRun::try_from(r).context("failed to create running task run")?;
                TaskAggregate::Running(RunningTaskAggregate { task, task_run })
            }
            TaskRunStatus::Failed => {
                let task = FailingTask::from_task_base(updated_task_base, now)?;
                let last_task_run =
                    FailedTaskRun::try_from(r).context("failed to create failed task run")?;
                TaskAggregate::Failing(FailingTaskAggregate {
                    task,
                    task_run: FailingTaskRun::Failed(last_task_run),
                })
            }
            TaskRunStatus::Dead => {
                let task = FailingTask::from_task_base(updated_task_base, now)?;
                let last_task_run =
                    DeadTaskRun::try_from(r).context("failed to create dead task run")?;
                TaskAggregate::Failing(FailingTaskAggregate {
                    task,
                    task_run: FailingTaskRun::Dead(last_task_run),
                })
            }
            TaskRunStatus::Aborted => {
                let task = HealthyTask::from_task_base(updated_task_base, now)?;
                let task_run =
                    AbortedTaskRun::try_from(r).context("failed to create aborted task run")?;
                TaskAggregate::Healthy(HealthyTaskAggregate {
                    task,
                    last_task_run: Some(HealthyTaskRun::Aborted(task_run)),
                })
            }
            TaskRunStatus::Finished => {
                let task = HealthyTask::from_task_base(updated_task_base, now)?;
                let task_run =
                    FinishedTaskRun::try_from(r).context("failed to create finished task run")?;
                TaskAggregate::Healthy(HealthyTaskAggregate {
                    task,
                    last_task_run: Some(HealthyTaskRun::Finished(task_run)),
                })
            }
        },
    };

    Ok(agg)
}

/// Converts a boundary task and its last task run into a task aggregate.
///
/// Arguments:
/// * `boundary_task`: The boundary task to convert.
/// * `boundary_task_run`: The last task run of the boundary task.
pub fn from_boundary(
    boundary_task: BoundaryTask,
    boundary_task_run: Option<BoundaryTaskRun>,
) -> anyhow::Result<TaskAggregate> {
    Ok(match boundary_task.status {
        TaskStatus::Healthy => TaskAggregate::Healthy(HealthyTaskAggregate {
            last_task_run: match boundary_task_run {
                Some(r) if r.status == TaskRunStatus::Finished => {
                    Some(HealthyTaskRun::Finished(r.try_into()?))
                }
                Some(r) if r.status == TaskRunStatus::Aborted => {
                    Some(HealthyTaskRun::Aborted(r.try_into()?))
                }
                Some(r) => anyhow::bail!(TaskAggregateError::InconsistentTaskRunState {
                    task_id: boundary_task.user_id.clone(),
                    task_run_status: r.status,
                    details: "invalid task run status for healthy task".to_string(),
                }),
                None => None,
            },
            task: boundary_task.try_into()?,
        }),
        TaskStatus::Failing => TaskAggregate::Failing(FailingTaskAggregate {
            task_run: match boundary_task_run {
                Some(r) if r.status == TaskRunStatus::Failed => {
                    FailingTaskRun::Failed(r.try_into()?)
                }
                Some(r) if r.status == TaskRunStatus::Dead => FailingTaskRun::Dead(r.try_into()?),
                Some(r) => anyhow::bail!(TaskAggregateError::InconsistentTaskRunState {
                    task_id: boundary_task.user_id.clone(),
                    task_run_status: r.status,
                    details: "invalid task run status for failing task".to_string(),
                }),
                None => anyhow::bail!("Missing task run for failing task"),
            },
            task: boundary_task.try_into()?,
        }),
        TaskStatus::Running => TaskAggregate::Running(RunningTaskAggregate {
            task: boundary_task.try_into()?,
            task_run: boundary_task_run
                .ok_or_else(|| anyhow::anyhow!("Missing task run for running task"))?
                .try_into()?,
        }),
        TaskStatus::Due => TaskAggregate::Due(DueTaskAggregate {
            task: boundary_task.try_into()?,
        }),
        TaskStatus::Late => TaskAggregate::Late(LateTaskAggregate {
            task: boundary_task.try_into()?,
        }),
        TaskStatus::Absent => TaskAggregate::Absent(AbsentTaskAggregate {
            task: boundary_task.try_into()?,
        }),
        TaskStatus::Archived => TaskAggregate::Archived(ArchivedTaskAggregate {
            task: boundary_task.try_into()?,
        }),
    })
}

/// Convert a `TaskAggregate` to a `BoundaryTask` and an optional `BoundaryTaskRun`.
/// This conversion, in turn, lets us serve the aggregate from the API, or save it to the database.
///
/// # Arguments
/// * `aggregate` - The `TaskAggregate` to convert.
pub fn to_boundary(
    aggregate: TaskAggregate,
) -> anyhow::Result<(BoundaryTask, Option<BoundaryTaskRun>)> {
    Ok(match aggregate {
        TaskAggregate::Due(d) => (d.task.try_into()?, None),
        TaskAggregate::Late(l) => (l.task.try_into()?, None),
        TaskAggregate::Running(r) => (r.task.try_into()?, Some(r.task_run.into())),
        TaskAggregate::Failing(f) => (f.task.try_into()?, Some(f.task_run.into())),
        TaskAggregate::Healthy(h) => (h.task.try_into()?, h.last_task_run.map(|lr| lr.into())),
        TaskAggregate::Absent(a) => (a.task.try_into()?, None),
        TaskAggregate::Archived(a) => (a.task.try_into()?, None),
    })
}
