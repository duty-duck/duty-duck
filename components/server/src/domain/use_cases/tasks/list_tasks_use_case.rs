use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use utoipa::{IntoParams, ToSchema};

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        task::{BoundaryTask, TaskStatus},
    },
    ports::task_repository::{TaskRepository, ListTasksOutput},
};

#[derive(Serialize, Deserialize, TS, Clone, Debug, IntoParams)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListTasksParams {
    #[serde(default)]
    pub status_filter: Vec<TaskStatus>,
    #[serde(default)]
    pub search_query: String,
    pub page_number: Option<u32>,
    pub items_per_page: Option<u32>,
}

#[derive(Serialize, TS, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListTasksResponse {
    pub items: Vec<BoundaryTask>,
    pub total_number_of_results: u32,
    pub total_number_of_filtered_results: u32,
}

#[derive(Error, Debug)]
pub enum ListTasksError {
    #[error("Failed to list tasks: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
    #[error("Current user doesn't have the privilege to list tasks")]
    Forbidden,
}

pub async fn list_tasks(
    auth_context: &AuthContext,
    repository: &impl TaskRepository,
    params: ListTasksParams,
) -> Result<ListTasksResponse, ListTasksError> {
    if !auth_context.can(Permission::ReadTasks) {
        return Err(ListTasksError::Forbidden);
    }

    let items_per_page = params.items_per_page.unwrap_or(10).min(50);
    let page_number = params.page_number.unwrap_or(1);

    let ListTasksOutput {
        tasks,
        total_tasks,
        total_filtered_tasks,
    } = repository
        .list_tasks(
            auth_context.active_organization_id,
            params.status_filter,
            params.search_query,
            items_per_page,
            items_per_page * (page_number - 1),
        )
        .await
        .map_err(ListTasksError::TechnicalFailure)?;

    Ok(ListTasksResponse {
        items: tasks,
        total_number_of_results: total_tasks,
        total_number_of_filtered_results: total_filtered_tasks,
    })
} 