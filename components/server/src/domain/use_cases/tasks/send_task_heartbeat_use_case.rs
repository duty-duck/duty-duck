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
pub enum SendTaskHeartbeatError {
    #[error("Task not found")]
    TaskNotFound,
    #[error("Task is not running")]
    TaskIsNotRunning,
    #[error("User is not allowed to send a heartbeat for this task")]
    Forbidden,
    #[error("Technical error")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn send_task_heartbeat_use_case<TR, TRR>(
    auth_context: &AuthContext,
    task_repository: &TR,
    task_run_repository: &TRR,
    task_id: TaskId,
) -> Result<(), SendTaskHeartbeatError>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
{
    if !auth_context.can(Permission::WriteTaskRuns) {
        return Err(SendTaskHeartbeatError::Forbidden);
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
        None => return Err(SendTaskHeartbeatError::TaskNotFound),
        Some(TaskAggregate::Running(t)) => {
            t.receive_heartbeat(now).context("failed to receive heartbeat")?
        },
        Some(_) => return Err(SendTaskHeartbeatError::TaskIsNotRunning),
    };

    save_task_aggregate(task_repository, task_run_repository, &mut tx, TaskAggregate::Running(running_aggregate)).await?;
    task_repository.commit_transaction(tx).await?;

    Ok(())
}
