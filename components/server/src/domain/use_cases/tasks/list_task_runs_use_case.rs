use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        task::TaskId,
        task_run::{BoundaryTaskRun, TaskRunStatus},
    },
    ports::task_run_repository::{ListTaskRunsOpts, ListTaskRunsOutput, TaskRunRepository},
};

#[derive(Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ListTaskRunsParams {
    #[serde(default)]
    pub include_statuses: Option<Vec<TaskRunStatus>>,
    #[serde(default)]
    pub page_number: Option<u32>,
    #[serde(default)]
    pub items_per_page: Option<u32>,
}

#[derive(Serialize, TS, ToSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ListTaskRunsResponse {
    pub runs: Vec<BoundaryTaskRun>,
    pub total_runs: u32,
    pub total_filtered_runs: u32,
}

#[derive(Error, Debug)]
pub enum ListTaskRunsError {
    #[error("User is not allowed to list task runs")]
    Forbidden,
    #[error("Technical failure occured while listing task runs")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn list_task_runs_use_case(
    auth_context: &AuthContext,
    task_run_repository: &impl TaskRunRepository,
    task_id: TaskId,
    params: ListTaskRunsParams,
) -> Result<ListTaskRunsResponse, ListTaskRunsError> {
    if !auth_context.can(Permission::ReadTaskRuns) {
        return Err(ListTaskRunsError::Forbidden);
    }

    let mut transaction = task_run_repository.begin_transaction().await?;
    let items_per_page = params.items_per_page.unwrap_or(15).min(50);
    let page_number = params.page_number.unwrap_or(1);
    let ListTaskRunsOutput { runs, total_runs, total_filtered_runs } = task_run_repository
        .list_task_runs(
            &mut transaction,
            auth_context.active_organization_id,
            ListTaskRunsOpts {
                task_id: &task_id,
                include_statuses: &params.include_statuses.unwrap_or_default(),
                limit: items_per_page,
                offset: (page_number - 1) * items_per_page,
            },
        )
        .await?;

    Ok(ListTaskRunsResponse { runs, total_runs, total_filtered_runs })
}
