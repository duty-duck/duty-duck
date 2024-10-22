use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use chrono::{Utc, DateTime};
use utoipa::{IntoParams, ToSchema};

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        incident::{Incident, IncidentPriority, IncidentStatus},
    },
    ports::incident_repository::{IncidentRepository, ListIncidentsOpts, ListIncidentsOutput}, use_cases::shared::OrderDirection,
};

/// Parameters for listing incidents
#[derive(Serialize, Deserialize, TS, Clone, Debug, IntoParams)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListIncidentsParams {
    pub page_number: Option<u32>,
    pub items_per_page: Option<u32>,
    pub status: Option<Vec<IncidentStatus>>,
    pub priority: Option<Vec<IncidentPriority>>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub order_by: Option<OrderIncidentsBy>,
    pub order_direction: Option<OrderDirection>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum OrderIncidentsBy {
    #[default]
    CreatedAt,
    Priority,
}

#[derive(Serialize, TS, Clone, Debug, ToSchema)]
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
            ListIncidentsOpts {
                include_statuses: &include_statuses,
                include_priorities: &include_priorities,
                include_sources: &[],
                limit: items_per_page,
                offset: items_per_page * (page_number - 1),
                from_date: params.from_date,
                to_date: params.to_date,
                order_by: params.order_by.unwrap_or(OrderIncidentsBy::CreatedAt),
                order_direction: params.order_direction.unwrap_or(OrderDirection::Desc),
            },
        )
        .await?;
    Ok(ListIncidentsResponse {
        items: incidents,
        total_number_of_filtered_results: total_filtered_incidents,
        total_number_of_results: total_incidents,
    })
}
