use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use chrono::{Utc, DateTime};

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        incident::{IncidentPriority, IncidentStatus, Incident},
    },
    ports::incident_repository::{IncidentRepository, ListIncidentsOutput},
};

#[derive(Serialize, Deserialize, TS, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListIncidentsParams {
    pub page_number: Option<u32>,
    pub items_per_page: Option<u32>,
    pub status: Option<Vec<IncidentStatus>>,
    pub priority: Option<Vec<IncidentPriority>>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

#[derive(Serialize, TS, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListIncidentsResponse {
    pub items: Vec<Incident>,
    pub total_number_of_results: u32,
    pub total_number_of_filtered_results: u32,
}

#[derive(Error, Debug)]
pub enum ListIncidentsError {
    #[error("Current user doesn't have the privilege the list incidents")]
    Forbidden,
    #[error("Failed to get incidents from the database: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn list_incidents(
    auth_context: &AuthContext,
    repository: &impl IncidentRepository,
    params: ListIncidentsParams,
) -> Result<ListIncidentsResponse, ListIncidentsError> {
    if !auth_context.can(Permission::ReadIncidents) {
        return Err(ListIncidentsError::Forbidden);
    }

    let items_per_page = params.items_per_page.unwrap_or(10).min(50);
    let page_number = params.page_number.unwrap_or(1);
    let include_statuses = params.status.unwrap_or(IncidentStatus::ALL.to_vec());
    let include_priorities = params.priority.unwrap_or(IncidentPriority::ALL.to_vec());
    let mut tx = repository.begin_transaction().await?;

    let ListIncidentsOutput {
        incidents,
        total_filtered_incidents,
        total_incidents,
    } = repository
        .list_incidents(
            &mut tx,
            auth_context.active_organization_id,
            &include_statuses,
            &include_priorities,
            &[],
            items_per_page,
            items_per_page * (page_number - 1),
            params.from_date,
            params.to_date,
        )
        .await?;
    Ok(ListIncidentsResponse {
        items: incidents,
        total_number_of_filtered_results: total_filtered_incidents,
        total_number_of_results: total_incidents,
    })
}
