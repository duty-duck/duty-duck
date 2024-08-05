use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        http_monitor::{HttpMonitor, HttpMonitorStatus},
    },
    ports::http_monitor_repository::{HttpMonitorRepository, ListHttpMonitorsOutput},
};

#[derive(Serialize, Deserialize, TS, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListHttpMonitorsParams {
    pub page_number: Option<u32>,
    pub items_per_page: Option<u32>,
    pub include: Option<Vec<HttpMonitorStatus>>,
    pub query: Option<String>,
}

#[derive(Serialize, TS, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListHttpMonitorsResponse {
    pub items: Vec<HttpMonitor>,
    pub total_number_of_results: u32,
    pub total_number_of_filtered_results: u32,
}

#[derive(Error, Debug)]
pub enum ListHttpMonitorsError {
    #[error("Current user doesn't have the privilege the list HTTP monitors")]
    Forbidden,
    #[error("Failed to get monitors from the database: {0}")]
    TechnicalError(#[from] anyhow::Error),
}

pub async fn list_http_monitors(
    auth_context: &AuthContext,
    repository: &impl HttpMonitorRepository,
    params: ListHttpMonitorsParams,
) -> Result<ListHttpMonitorsResponse, ListHttpMonitorsError> {
    if !auth_context.can(Permission::ReadHttpMonitors) {
        return Err(ListHttpMonitorsError::Forbidden);
    }

    let items_per_page = params.items_per_page.unwrap_or(10).min(50);
    let page_number = params.page_number.unwrap_or(1);
    let include_statuses = params.include.unwrap_or(HttpMonitorStatus::ALL.to_vec());

    let ListHttpMonitorsOutput {
        monitors,
        total_filtered_monitors,
        total_monitors,
    } = repository
        .list_http_monitors(
            auth_context.active_organization_id,
            include_statuses,
            params.query.unwrap_or_default(),
            items_per_page,
            items_per_page * (page_number - 1),
        )
        .await?;
    Ok(ListHttpMonitorsResponse {
        items: monitors,
        total_number_of_filtered_results: total_filtered_monitors,
        total_number_of_results: total_monitors,
    })
}
