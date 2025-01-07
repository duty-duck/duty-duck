mod absent;
mod due;
mod failing;
mod healthy;
mod late;
mod running;

pub use absent::*;
use anyhow::Context;
pub use due::*;
pub use failing::*;
pub use healthy::*;
pub use late::*;
pub use running::*;

use crate::domain::{
    entities::{task::*, task_run::*},
    ports::{
        task_repository::TaskRepository,
        task_run_repository::TaskRunRepository,
    },
};
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

/// Retrieve a task aggregate from the database by its id
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
                        .await?;

                    from_boundary(task, task_run)?
                }
                TaskStatus::Failing => {
                    let task_run = task_run_repository
                        .get_latest_task_run(
                            tx,
                            organization_id,
                            task_id,
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
                            task_id,
                            &[TaskRunStatus::Aborted, TaskRunStatus::Finished],
                        )
                        .await?;

                    from_boundary(task, last_task_run)?
                }
            };

            Ok(Some(aggregate))
        }
        None => Ok(None),
    }
}

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

pub fn from_boundary(
    boundary_task: BoundaryTask,
    boundary_task_run: Option<BoundaryTaskRun>,
) -> anyhow::Result<TaskAggregate> {
    Ok(match boundary_task.status {
        TaskStatus::Healthy => TaskAggregate::Healthy(HealthyTaskAggregate {
            last_task_run: match boundary_task_run {
                Some(r) if r.status == TaskRunStatus::Finished => Some(HealthyTaskRun::Finished(r.try_into()?)),
                Some(r) if r.status == TaskRunStatus::Aborted => Some(HealthyTaskRun::Aborted(r.try_into()?)),
                Some(r) => anyhow::bail!(TaskAggregateError::InconsistentTaskRunState {
                    task_id: boundary_task.id.clone(),
                    task_run_status: r.status,
                    details: "invalid task run status for healthy task".to_string(),
                }),
                None => None,
            },
            task: boundary_task.try_into()?,
        }),
        TaskStatus::Failing => TaskAggregate::Failing(FailingTaskAggregate {
            task_run: match boundary_task_run {
                Some(r) if r.status == TaskRunStatus::Failed => FailingTaskRun::Failed(r.try_into()?),
                Some(r) if r.status == TaskRunStatus::Dead => FailingTaskRun::Dead(r.try_into()?),
                Some(r) => anyhow::bail!(TaskAggregateError::InconsistentTaskRunState {
                    task_id: boundary_task.id.clone(),
                    task_run_status: r.status,
                    details: "invalid task run status for failing task".to_string(),
                }),
                None => anyhow::bail!("Missing task run for failing task"),
            },
            task: boundary_task.try_into()?,
        }),
        TaskStatus::Running => TaskAggregate::Running(RunningTaskAggregate {
            task: boundary_task.try_into()?,
            task_run: boundary_task_run.ok_or_else(|| anyhow::anyhow!("Missing task run for running task"))?.try_into()?,
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
    })
}

pub fn to_boundary(
    aggregate: TaskAggregate,
) -> anyhow::Result<(BoundaryTask, Option<BoundaryTaskRun>)> {
    Ok(match aggregate {
        TaskAggregate::Due(d) => (d.task.try_into()?, None),
        TaskAggregate::Late(l) => (l.task.try_into()?, None),
        TaskAggregate::Running(r) => (r.task.try_into()?, Some(r.task_run.into())),
        TaskAggregate::Failing(f) => (f.task.try_into()?, Some(f.task_run.into())),
        TaskAggregate::Healthy(h) => (
            h.task.try_into()?,
            h.last_task_run.map(|lr| lr.into()),
        ),
        TaskAggregate::Absent(a) => (a.task.try_into()?, None),
    })
}
