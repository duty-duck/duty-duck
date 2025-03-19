use std::time::Duration;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        task::TaskId,
        task_run::{IndexedTaskRunLogEvent, TaskRunLogEvent},
    },
    ports::{
        logs::{LogsSearcher, SearchLogsOpts, SearchLogsQuery, SearchLogsQueryClause},
        task_repository::TaskRepository,
        task_run_repository::TaskRunRepository,
    },
};

#[derive(Error, Debug)]
pub enum GetTaskRunLogsError {
    #[error("Task or task run not found")]
    NotFound,
    #[error("User is not allowed to retreive logs for this task run")]
    Forbidden,
    #[error("Technical error: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, IntoParams)]
#[ts(export)]
pub struct GetTaskRunLogsParams {
    #[serde(default)]
    /// How many log records to retrieve at a time. The default value is 200, the maximum value is 500
    #[ts(type = "number | null")]
    pub limit: Option<u64>,
    #[ts(type = "number | null")]
    pub offset: Option<u64>,
}

#[derive(Serialize, Deserialize, TS, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GetTaskRunLogsResponse {
    pub total_number_of_logs: u64,
    pub logs: Vec<IndexedTaskRunLogEvent>,
}

pub async fn get_task_run_logs_use_case<TR, TRR, LS>(
    auth_context: &AuthContext,
    task_repository: &TR,
    task_run_repository: &TRR,
    logs_searcher: &LS,
    task_id: TaskId,
    task_run_id: Uuid,
    params: GetTaskRunLogsParams,
) -> Result<GetTaskRunLogsResponse, GetTaskRunLogsError>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
    LS: LogsSearcher,
{
    if !auth_context.can(Permission::ReadTaskRuns) {
        return Err(GetTaskRunLogsError::Forbidden);
    }

    let mut tx = task_repository.begin_transaction().await?;

    let task = task_repository
        .get_task_by_id(&mut tx, auth_context.active_organization_id()?, &task_id)
        .await
        .context("Failed to get task from repository")?
        .ok_or(GetTaskRunLogsError::NotFound)?;

    let task_run = task_run_repository
        .get_task_run(&mut tx, auth_context.active_organization_id()?, task_run_id)
        .await
        .context("Failed to get task run from repository")?
        .ok_or(GetTaskRunLogsError::NotFound)?;

    if task_run.task_id != task.id {
        return Err(GetTaskRunLogsError::NotFound);
    }

    // abort the transaction ASAP
    drop(tx);

    let offset = params.offset.unwrap_or_default();
    let limit = params.limit.unwrap_or(200).min(500);
    let task_run_id_str = task_run_id.to_string();
    let output = logs_searcher
        .search_logs(
            auth_context.active_organization_id()?,
            SearchLogsOpts {
                // we can use the task run's timestamps as bondaries to retrieve logs, so the query is more efficient
                from_timestamp: Some(task_run.started_at),
                to_timestamp: task_run
                    .completed_at
                    .map(|timestamp| timestamp + Duration::from_secs(2)),
                start_offset: offset,
                max_hits: limit,
                query: SearchLogsQuery::And(
                    Box::new(SearchLogsQuery::Clause(SearchLogsQueryClause::Term {
                        field_name: "resource_attributes.service.namespace",
                        term: "dutyduck.Task",
                    })),
                    Box::new(SearchLogsQuery::Clause(SearchLogsQueryClause::Term {
                        field_name: "resource_attributes.service.instance.id",
                        term: &task_run_id_str,
                    })),
                ),
            },
        )
        .await?;

    Ok(GetTaskRunLogsResponse {
        total_number_of_logs: output.total_hits,
        logs: output
            .logs
            .into_iter()
            .map(TaskRunLogEvent::from)
            .enumerate()
            .map(|(ix, event)| IndexedTaskRunLogEvent {
                index: (ix as u64) + offset,
                event,
            })
            .collect(),
    })
}
