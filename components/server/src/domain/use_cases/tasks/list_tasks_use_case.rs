use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use utoipa::{IntoParams, ToSchema};

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission}, entity_metadata::MetadataFilter, task::{BoundaryTask, TaskStatus}
    },
    ports::task_repository::{ListTasksOpts, ListTasksOutput, TaskRepository}, use_cases::shared::OrderDirection,
};

#[derive(Serialize, Deserialize, TS, Clone, Debug, IntoParams)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListTasksParams {
    #[serde(default)]
    pub include: Option<Vec<TaskStatus>>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub page_number: Option<u32>,
    #[serde(default)]
    pub items_per_page: Option<u32>,
    #[serde(default)]
    pub order_by: Option<OrderTasksBy>,
    #[serde(default)]
    pub order_direction: Option<OrderDirection>,
    #[ts(type = "Option<MetadataFilter>")]
    pub metadata_filter: Option<String>,
}

impl ListTasksParams {
    pub fn metadata_filter(&self) -> MetadataFilter {
        self.metadata_filter
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum OrderTasksBy {
    #[default]
    CreatedAt,
    LastStatusChangeAt,
    Name
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
            ListTasksOpts {
                metadata_filter: params.metadata_filter(),
                include_statuses: &params.include.unwrap_or_default(),
                query: &params.query.unwrap_or_default(),
                limit: items_per_page,
                offset: items_per_page * (page_number - 1),
                order_by: params.order_by.unwrap_or(OrderTasksBy::LastStatusChangeAt),
                order_direction: params.order_direction.unwrap_or(OrderDirection::Desc),
            },
        )
        .await
        .map_err(ListTasksError::TechnicalFailure)?;

    Ok(ListTasksResponse {
        items: tasks,
        total_number_of_results: total_tasks,
        total_number_of_filtered_results: total_filtered_tasks,
    })
} 