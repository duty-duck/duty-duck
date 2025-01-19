use serde::Deserialize;
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        entity_metadata::EntityMetadata,
        task::{BoundaryTask, HealthyTask, TaskError, TaskId},
    },
    ports::task_repository::TaskRepository,
};

#[derive(Error, Debug)]
pub enum CreateTaskError {
    #[error("User is not allowed to create a task")]
    Forbidden,
    #[error("Task with id {0} already exists")]
    TaskAlreadyExists(TaskId),
    #[error("Technical failure occured while creating a task")]
    TechnicalFailure(#[from] anyhow::Error),
    #[error("Invalid cron schedule: {details}")]
    InvalidCronSchedule { details: cron::error::Error },
    #[error("Technical failure occured while creating a task")]
    TaskError(#[from] TaskError),
}

#[derive(Debug, Deserialize, TS, ToSchema, Clone)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskCommand {
    #[ts(type = "string")]
    pub id: TaskId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub cron_schedule: Option<String>,
    pub schedule_timezone: Option<String>,
    pub start_window_seconds: Option<u32>,
    pub lateness_window_seconds: Option<u32>,
    pub heartbeat_timeout_seconds: Option<u32>,
    pub metadata: Option<EntityMetadata>,
}

pub async fn create_task_use_case(
    auth_context: &AuthContext,
    task_repository: &impl TaskRepository,
    command: CreateTaskCommand,
) -> Result<(), CreateTaskError> {
    if !auth_context.can(Permission::WriteTasks) {
        return Err(CreateTaskError::Forbidden);
    }

    let mut tx = task_repository.begin_transaction().await?;

    let existing_task = task_repository
        .get_task_by_user_id(&mut tx, auth_context.active_organization_id, &command.id)
        .await?;

    if existing_task.is_some() {
        return Err(CreateTaskError::TaskAlreadyExists(command.id));
    }

    let new_task = HealthyTask::new(auth_context.active_organization_id, command)?;
    let new_task: BoundaryTask = new_task.try_into().map_err(|e| match e {
        TaskError::InvalidCronSchedule { details } => {
            CreateTaskError::InvalidCronSchedule { details }
        }
        _ => CreateTaskError::TechnicalFailure(e.into()),
    })?;
    task_repository.upsert_task(&mut tx, new_task).await?;
    task_repository.commit_transaction(tx).await?;

    Ok(())
}
