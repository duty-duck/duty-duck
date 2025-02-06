use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        task::TaskId,
        task_run::BoundaryTaskRun,
    },
    ports::{task_repository::TaskRepository, task_run_repository::TaskRunRepository},
};

#[derive(Serialize, TS, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GetTaskRunResponse {
    pub task_run: BoundaryTaskRun,
}

#[derive(Error, Debug)]
pub enum GetTaskRunError {
    #[error("Failed to get task run: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
    #[error("Current user doesn't have the privilege to read task runs")]
    Forbidden,
    #[error("Task or task run not found")]
    NotFound,
}

pub async fn get_task_run<
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
>(
    auth_context: &AuthContext,
    task_repository: &TR,
    task_run_repository: &TRR,
    task_id: TaskId,
    task_run_started_at: DateTime<Utc>,
) -> Result<GetTaskRunResponse, GetTaskRunError> {
    if !auth_context.can(Permission::ReadTaskRuns) {
        return Err(GetTaskRunError::Forbidden);
    }

    let mut tx = task_repository
        .begin_transaction()
        .await
        .context("Failed to begin transaction")?;

    let task = task_repository
        .get_task_by_id(&mut tx, auth_context.active_organization_id, &task_id)
        .await
        .context("Failed to get task from repository")?
        .ok_or(GetTaskRunError::NotFound)?;

    let task_run = task_run_repository
        .get_task_run(
            &mut tx,
            auth_context.active_organization_id,
            task.id,
            task_run_started_at,
        )
        .await
        .context("Failed to get task run from repository")?
        .ok_or(GetTaskRunError::NotFound)?;

    Ok(GetTaskRunResponse { task_run })
}
