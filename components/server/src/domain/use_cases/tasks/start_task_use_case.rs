use anyhow::Context;
use chrono::Utc;
use serde::Deserialize;
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        task::{
            get_task_aggregate, save_task_aggregate, HealthyTaskAggregate,
            RunningTaskAggregate, TaskAggregate, TaskId,
        },
    },
    ports::{task_repository::TaskRepository, task_run_repository::TaskRunRepository},
};

use super::CreateTaskCommand;

/// An optional command that can be used to create a task on-the-fly when starting a task run
#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StartTaskCommand {
    /// The properties of the new task to create if the task does not exist yet
    #[serde(default)]
    pub new_task: Option<NewTask>,
    /// Whether to abort the previous running task
    #[serde(default)]
    pub abort_previous_running_task: bool,
}

#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NewTask {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cron_schedule: Option<String>,
    pub start_window_seconds: Option<u32>,
    pub lateness_window_seconds: Option<u32>,
    pub heartbeat_timeout_seconds: Option<u32>,
}

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
    command: Option<StartTaskCommand>,
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
    .await
    .context("failed to get task aggregate from the database")?;

    let now = Utc::now();

    let running_aggregate: RunningTaskAggregate = match aggregate {
        None => {
            let command = command.ok_or(StartTaskError::TaskNotFound)?;
            let new_task = command.new_task.ok_or(StartTaskError::TaskNotFound)?;
            let new_task = CreateTaskCommand {
                id: task_id,
                name: new_task.name,
                description: new_task.description,
                cron_schedule: new_task.cron_schedule,
                start_window_seconds: new_task.start_window_seconds,
                lateness_window_seconds: new_task.lateness_window_seconds,
                heartbeat_timeout_seconds: new_task.heartbeat_timeout_seconds,
            };
            let new_task = HealthyTaskAggregate::new(auth_context.active_organization_id, new_task)
                .context("failed to create a new task")?;
            new_task.start(now).context("failed to start new task")?.0
        }
        Some(TaskAggregate::Running(t)) => {
            if command.is_some_and(|c| c.abort_previous_running_task) {
                let aborted_task = t.mark_aborted(now).context("failed to abort running task")?;
                save_task_aggregate(
                    task_repository,
                    task_run_repository,
                    &mut tx,
                    TaskAggregate::Healthy(aborted_task.clone()),
                )
                .await
                .context("failed to save aborted task to the database")?;

                aborted_task.start(now).context("failed to start aborted task")?.0
            } else {
                return Err(StartTaskError::TaskAlreadyStarted);
            }
        }
        Some(TaskAggregate::Due(t)) => t.start(now).context("failed to start due task")?,
        Some(TaskAggregate::Late(t)) => t.start(now).context("failed to start late task")?,
        Some(TaskAggregate::Failing(t)) => t.start(now).context("failed to start failing task")?.0,
        Some(TaskAggregate::Healthy(t)) => t.start(now).context("failed to start healthy task")?.0,
        Some(TaskAggregate::Absent(t)) => t.start(now).context("failed to start absent task")?,
    };

    save_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        TaskAggregate::Running(running_aggregate),
    )
    .await
    .context("failed to save task aggregate to the database")?;

    task_repository.commit_transaction(tx).await.context("failed to commit transaction")?;

    Ok(())
}
