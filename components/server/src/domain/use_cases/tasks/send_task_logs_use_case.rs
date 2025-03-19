use anyhow::Context;
use chrono::Utc;
use serde::Deserialize;
use thiserror::Error;
use utoipa::ToSchema;

use crate::domain::{
    entities::{
        authorization::{AuthContext, OriginalAuthenticationToken, Permission},
        task::{
            get_task_aggregate, save_task_aggregate, RunningTaskAggregate, TaskAggregate, TaskId,
        },
        task_run::{TaskRunLogEvent, TaskRunLogEvents},
    },
    ports::{
        logs::LogsIngestor, task_repository::TaskRepository, task_run_repository::TaskRunRepository,
    },
};

#[derive(Error, Debug)]
pub enum SendTaskLogsError {
    #[error("Task not found")]
    TaskNotFound,
    #[error("Task is not running")]
    TaskIsNotRunning,
    #[error("User is not allowed to send a heartbeat for this task")]
    Forbidden,
    #[error("This endpoint does not support Bearer token authentication. Please use an API Token instead")]
    InvalidAuthenticationMethod,
    #[error("Technical error")]
    TechnicalFailure(#[from] anyhow::Error),
}

/// A request to store the logs associated with a running task so they can be consulted and searched.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SendTaskLogsRequest {
    events: Vec<TaskRunLogEvent>,
}

pub async fn send_task_logs_use_case<TR, TRR, LI>(
    auth_context: &AuthContext,
    task_repository: &TR,
    task_run_repository: &TRR,
    logs_ingestor: &LI,
    task_id: TaskId,
    request: SendTaskLogsRequest,
) -> Result<(), SendTaskLogsError>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
    LI: LogsIngestor,
{
    if !auth_context.can(Permission::WriteTaskRuns) {
        return Err(SendTaskLogsError::Forbidden);
    }

    if !matches!(
        auth_context.original_auth_token,
        Some(OriginalAuthenticationToken::APITokenPair { .. })
    ) {
        return Err(SendTaskLogsError::InvalidAuthenticationMethod);
    }

    let mut tx = task_repository.begin_transaction().await?;
    let aggregate = get_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        auth_context.active_organization_id()?,
        &task_id,
    )
    .await?;
    let now = Utc::now();

    let running_aggregate: RunningTaskAggregate = match aggregate {
        None => return Err(SendTaskLogsError::TaskNotFound),
        Some(TaskAggregate::Running(t)) =>
        // sending a log also counts as sending a hearbeat (it's a signal that the task is alive and well)
        {
            t.receive_heartbeat(now)
                .context("failed to receive heartbeat")?
        }
        Some(_) => return Err(SendTaskLogsError::TaskIsNotRunning),
    };

    let logs = TaskRunLogEvents {
        task_aggregate: &running_aggregate,
        log_events: request.events,
    }
    .into();

    save_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        TaskAggregate::Running(running_aggregate),
    )
    .await?;

    // commit the transaction before ingesting the log to avoid keeping the transaction open during the ingestion
    task_repository.commit_transaction(tx).await?;

    logs_ingestor.ingest_logs(auth_context, vec![logs]).await?;

    Ok(())
}
