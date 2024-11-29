use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        task::{BoundaryTask, TaskId},
    },
    ports::task_repository::TaskRepository,
};

#[derive(Serialize, TS, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GetTaskResponse {
    pub task: BoundaryTask,
}

#[derive(Error, Debug)]
pub enum GetTaskError {
    #[error("Failed to get task: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
    #[error("Current user doesn't have the privilege to read tasks")]
    Forbidden,
    #[error("Task not found")]
    NotFound,
}

pub async fn get_task(
    auth_context: &AuthContext,
    repository: &impl TaskRepository,
    task_id: TaskId,
) -> Result<GetTaskResponse, GetTaskError> {
    if !auth_context.can(Permission::ReadTasks) {
        return Err(GetTaskError::Forbidden);
    }

    let mut tx = repository
        .begin_transaction()
        .await
        .map_err(GetTaskError::TechnicalFailure)?;

    let task = repository
        .get_task(
            &mut tx,
            auth_context.active_organization_id,
            task_id,
        )
        .await
        .map_err(GetTaskError::TechnicalFailure)?
        .ok_or(GetTaskError::NotFound)?;

    repository
        .commit_transaction(tx)
        .await
        .map_err(GetTaskError::TechnicalFailure)?;

    Ok(GetTaskResponse { task })
} 