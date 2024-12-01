use anyhow::Context;
use chrono::Utc;
use thiserror::Error;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        task::{get_task_aggregate, save_task_aggregate, RunningTaskAggregate, TaskAggregate, TaskId},
    },
    ports::{task_repository::TaskRepository, task_run_repository::TaskRunRepository},
};

#[derive(Error, Debug)]
pub enum StartTaskError {
    #[error("Task not found")]
    TaskNotFound,
    #[error("Task already started")]
    TaskAlreadyStarted,
    #[error("User is not allowed to start this task")]
    Forbidden,
    #[error("Technical error")]
    TechnicalError(#[from] anyhow::Error),
}

pub async fn start_task_use_case<TR, TRR>(
    auth_context: &AuthContext,
    task_repository: &TR,
    task_run_repository: &TRR,
    task_id: TaskId,
) -> Result<(), StartTaskError>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
{
    if !auth_context.can(Permission::WriteTaskRuns) {
        return Err(StartTaskError::Forbidden);
    }

    let mut tx = task_repository.begin_transaction().await?;
    let aggregate = get_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        auth_context.active_organization_id,
        &task_id,
    )
    .await?;
    let now = Utc::now();

    let running_aggregate: RunningTaskAggregate = match aggregate {
        None => return Err(StartTaskError::TaskNotFound),
        Some(TaskAggregate::Running(_)) => return Err(StartTaskError::TaskAlreadyStarted),
        Some(TaskAggregate::Pending(t)) => t.start(now).context("failed to start pending task")?,
        Some(TaskAggregate::Due(t)) => t.start(now).context("failed to start due task")?,
        Some(TaskAggregate::Late(t)) => t.start(now).context("failed to start late task")?,
        Some(TaskAggregate::Failing(t)) => t.start(now).context("failed to start failing task")?.0,
        Some(TaskAggregate::Healthy(t)) => t.start(now).context("failed to start healthy task")?.0,
        Some(TaskAggregate::Absent(t)) => t.start(now).context("failed to start absent task")?,
    };

    save_task_aggregate(task_repository, task_run_repository, &mut tx, TaskAggregate::Running(running_aggregate)).await?;
    task_repository.commit_transaction(tx).await?;

    Ok(())
}
