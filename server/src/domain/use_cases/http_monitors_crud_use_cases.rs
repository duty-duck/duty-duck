use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

use crate::domain::{
    entities::{authorization::{AuthContext, Permission}, http_monitor::HttpMonitor},
    ports::http_monitor_repository::HttpMonitorRepository,
};

#[derive(Serialize, TS, Clone, Debug)]
pub struct ListHttpMonitorsResponse {
    pub items: Vec<HttpMonitor>,
    pub total_number_of_results: u64,
}

#[derive(Error, Debug)]
pub enum ListHttpMonitorsError {
    #[error("Current user doesn't have the privilege the list HTTP monitors")]
    Forbidden,
    #[error("Failed to get monitors from the database: {0}")]
    TechnicalError(#[from] anyhow::Error)
}

pub async fn list_http_monitors(
    auth_context: &AuthContext,
    repository: &impl HttpMonitorRepository,
    page_number: u32,
    items_per_page: u32,
) -> Result<ListHttpMonitorsResponse, ListHttpMonitorsError> {
    if !auth_context.can(Permission::ReadHttpMonitors) {
        return Err(ListHttpMonitorsError::Forbidden)
    }

    let items_per_page = items_per_page.min(50);
    let (items, total_number_of_results) = repository
        .list_http_monitors(
            auth_context.active_organization_id,
            items_per_page,
            items_per_page * (page_number - 1),
        )
        .await?;
    Ok(ListHttpMonitorsResponse {
        items,
        total_number_of_results,
    })
}
