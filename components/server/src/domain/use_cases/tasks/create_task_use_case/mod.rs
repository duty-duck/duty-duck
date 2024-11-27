use anyhow::Context;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;

#[cfg(test)]
mod tests;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        task::{TaskId, TaskStatus},
    },
    ports::task_repository::{TaskRepository, NewTask},
};

#[derive(Deserialize, TS, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateTaskCommand {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub cron_schedule: Option<String>,
    pub start_window_seconds: i32,
    pub lateness_window_seconds: i32,
    pub heartbeat_timeout_seconds: i32,
}

#[derive(Serialize, TS, Clone, Debug)]
#[ts(export)]
pub struct CreateTaskResponse {
    #[ts(type = "string")]
    pub id: TaskId,
}

#[derive(Error, Debug)]
pub enum CreateTaskError {
    #[error("Failed to create a task: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
    #[error("Current user doesn't have the privilege to create tasks")]
    Forbidden,
    #[error("Invalid task ID format")]
    InvalidTaskId,
    #[error("Invalid cron expression: {0}")]
    InvalidCronExpression(String),
}

pub async fn create_task(
    auth_context: &AuthContext,
    repository: &impl TaskRepository,
    command: CreateTaskCommand,
) -> Result<CreateTaskResponse, CreateTaskError> {
    if !auth_context.can(Permission::WriteTasks) {
        return Err(CreateTaskError::Forbidden);
    }

    // Validate task ID
    let task_id = TaskId::new(command.id)
        .ok_or(CreateTaskError::InvalidTaskId)?;

    // Validate cron expression if provided
    if let Some(cron_str) = &command.cron_schedule {
        croner::Cron::new(cron_str)
            .parse()
            .map_err(|e| CreateTaskError::InvalidCronExpression(e.to_string()))?;
    }

    let next_due_at = if command.is_active {
        if let Some(cron) = &command.cron_schedule {
            // Parse cron and get next execution time
            let cron = croner::Cron::new(cron)
                .parse()
                .map_err(|e| CreateTaskError::InvalidCronExpression(e.to_string()))?;
            let next_occurrence = cron.find_next_occurrence(&Utc::now(), true).context(
                "Failed to find next occurence for cron expression"
            )?;
            Some(next_occurrence)
        } else {
            None
        }
    } else {
        None
    };

    let new_task = NewTask {
        organization_id: auth_context.active_organization_id,
        id: task_id.clone(),
        name: command.name,
        description: command.description,
        status: if command.is_active {
            TaskStatus::Pending
        } else {
            TaskStatus::Inactive
        },
        cron_schedule: command.cron_schedule,
        next_due_at,
        start_window_seconds: command.start_window_seconds,
        lateness_window_seconds: command.lateness_window_seconds,
        heartbeat_timeout_seconds: command.heartbeat_timeout_seconds,
    };

    repository.create_task(new_task).await
        .map_err(CreateTaskError::TechnicalFailure)
        .map(|id| CreateTaskResponse { id })
}