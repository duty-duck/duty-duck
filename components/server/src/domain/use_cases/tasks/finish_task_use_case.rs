use anyhow::Context;
use chrono::Utc;
use serde::Deserialize;
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        task::{get_task_aggregate, save_task_aggregate, TaskAggregate, TaskId},
    },
    ports::{task_repository::TaskRepository, task_run_repository::TaskRunRepository},
};

#[derive(Error, Debug)]
pub enum FinishTaskError {
    #[error("User is not allowed to finish this task")]
    Forbidden,
    #[error("Task not found")]
    NotFound,
    #[error("Task is not running")]
    TaskIsNotRunning,
    #[error("Technical failure occured while finishing a task")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum FinishedTaskStatus {
    Success,
    Failure,
    Aborted,
}

#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FinishTaskCommand {
    pub status: FinishedTaskStatus,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub error_message: Option<String>,
}

pub async fn finish_task_use_case<TR, TRR>(
    auth_context: &AuthContext,
    task_repository: &TR,
    task_run_repository: &TRR,
    task_id: TaskId,
    command: FinishTaskCommand,
) -> Result<(), FinishTaskError>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
{
    if !auth_context.can(Permission::WriteTaskRuns) {
        return Err(FinishTaskError::Forbidden);
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
    let updated_aggregate = match aggregate {
        None => return Err(FinishTaskError::NotFound),
        Some(TaskAggregate::Running(t)) => match command.status {
            FinishedTaskStatus::Success => TaskAggregate::Healthy(
                t.mark_finished(now, command.exit_code)
                    .context("failed to finish running task")?,
            ),
            FinishedTaskStatus::Failure => TaskAggregate::Failing(
                t.mark_failed(now, command.exit_code, command.error_message)
                    .context("failed to finish running task")?,
            ),
            FinishedTaskStatus::Aborted => TaskAggregate::Healthy(
                t.mark_aborted(now)
                    .context("failed to finish running task")?,
            ),
        },
        Some(_) => return Err(FinishTaskError::TaskIsNotRunning),
    };

    save_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        updated_aggregate,
    )
    .await?;
    task_repository.commit_transaction(tx).await?;

    Ok(())
}
