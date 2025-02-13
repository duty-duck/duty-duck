use anyhow::Context;
use chrono::Utc;
use serde::Deserialize;

use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        entity_metadata::EntityMetadata,
        task::{
            get_task_aggregate, save_task_aggregate, update_task_aggregate, TaskError, TaskId,
            TaskUserId,
        },
    },
    ports::{task_repository::TaskRepository, task_run_repository::TaskRunRepository},
};

#[derive(Error, Debug)]
pub enum UpdateTaskError {
    #[error("Task does not exist")]
    TaskNotFound,
    #[error("User is not allowed to update a task")]
    Forbidden,
    #[error("User id conflicting with existing task")]
    UserIdConflict,
    #[error("Task is archived, it cannot be updated")]
    TaskArchived,
    #[error("Invalid task update")]
    TaskError(#[from] TaskError),
    #[error("Technical failure occured while creating a task")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Deserialize, TS, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateTaskCommand {
    #[ts(type = "string")]
    /// A unique, user-friendly identifier for the task.
    /// Will keep the id of the existing task if null
    pub user_id: Option<TaskUserId>,
    /// The name of the task.
    /// Will keep the name of the existing task if null
    pub name: Option<String>,
    /// A description of the task.
    /// Will remove the description of the existing task if null
    pub description: Option<String>,
    /// A cron schedule for the task, to make it a scheduled task
    /// Will erase the cron_schedule of the exisint task if null
    pub cron_schedule: Option<String>,
    /// A timezone for the task, if this is a scheduled task
    /// Will reset the timezone to UTC if null
    pub schedule_timezone: Option<String>,
    /// A number of seconds to wait, once this scheduled task is due, before the task is considered late
    /// Will reset to the default value if null
    pub start_window_seconds: Option<u32>,
    /// A number of seconds to wait, once this scheduled task is late, before the task is considered failed
    /// Will reset to the default value if null
    pub lateness_window_seconds: Option<u32>,
    /// A number of seconds to wait, after the last heartbeat was received, if any, before the task is considered failed
    /// Will reset to the default value if null
    pub heartbeat_timeout_seconds: Option<u32>,
    /// Metadata used to organize and filter tasks
    /// Will keep the metadata of the existing task if null
    pub metadata: Option<EntityMetadata>,
    /// Whether to send an email notification when an incident occurs for this task
    /// Will keep the existing value if null
    pub email_notification_enabled: Option<bool>,
    /// Whether to send a push notification when an incident occurs for this task
    /// Will keep the existing value if null
    pub push_notification_enabled: Option<bool>,
    /// Whether to send a SMS notification when an incident occurs for this task
    /// Will keep the existing value if null
    pub sms_notification_enabled: Option<bool>,
}

#[tracing::instrument(skip(auth_context, task_repository, task_run_repository))]
pub async fn update_task<TR, TRR>(
    auth_context: &AuthContext,
    task_repository: &TR,
    task_run_repository: &TRR,
    task_id: TaskId,
    command: UpdateTaskCommand,
) -> Result<(), UpdateTaskError>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
{
    if !auth_context.can(Permission::WriteTasks) {
        return Err(UpdateTaskError::Forbidden);
    }

    let mut tx = task_repository.begin_transaction().await?;
    let now = Utc::now();

    let aggregate = get_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        auth_context.active_organization_id,
        &task_id,
    )
    .await?
    .ok_or(UpdateTaskError::TaskNotFound)?;

    let new_aggregate = update_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        aggregate,
        command,
        now,
    )
    .await?;

    save_task_aggregate(task_repository, task_run_repository, &mut tx, new_aggregate)
        .await
        .context("Failed to persist updated task aggregate")?;

    task_repository
        .commit_transaction(tx)
        .await
        .context("Failed to commit transaction")?;

    Ok(())
}
