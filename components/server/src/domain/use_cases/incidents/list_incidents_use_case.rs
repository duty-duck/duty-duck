use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use thiserror::Error;
use ts_rs::TS;
use utoipa::{IntoParams, ToSchema};

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        entity_metadata::MetadataFilter,
        incident::{Incident, IncidentPriority, IncidentStatus, IncidentWithUsers},
    },
    ports::{
        incident_repository::{IncidentRepository, ListIncidentsOpts, ListIncidentsOutput},
        user_repository::UserRepository,
    },
    use_cases::shared::OrderDirection,
};

/// Parameters for listing incidents
#[serde_as]
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
    #[ts(type = "MetadataFilter | null")]
    pub metadata_filter: Option<String>,
}

impl ListIncidentsParams {
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
pub enum OrderIncidentsBy {
    #[default]
    CreatedAt,
    Priority,
}

#[derive(Serialize, TS, Clone, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListIncidentsResponse {
    pub items: Vec<IncidentWithUsers>,
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
    incident_repository: &impl IncidentRepository,
    user_repository: &impl UserRepository,
    params: ListIncidentsParams,
) -> Result<ListIncidentsResponse, ListIncidentsError> {
    if !auth_context.can(Permission::ReadIncidents) {
        return Err(ListIncidentsError::Forbidden);
    }

    let metadata_filter = params.metadata_filter();
    let items_per_page = params.items_per_page.unwrap_or(10).min(50);
    let page_number = params.page_number.unwrap_or(1);
    let include_statuses = params.status.unwrap_or(IncidentStatus::ALL.to_vec());
    let include_priorities = params.priority.unwrap_or(IncidentPriority::ALL.to_vec());
    let mut tx = incident_repository.begin_transaction().await?;

    let ListIncidentsOutput {
        incidents,
        total_filtered_incidents,
        total_incidents,
    } = incident_repository
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
                metadata_filter,
            },
        )
        .await?;
    Ok(ListIncidentsResponse {
        items: enrich_incidents_with_users(incidents, user_repository).await?,
        total_number_of_filtered_results: total_filtered_incidents,
        total_number_of_results: total_incidents,
    })
}

/// Enriches an incident with the user information
pub async fn enrich_incident_with_users(
    incident: Incident,
    user_repository: &impl UserRepository,
) -> Result<IncidentWithUsers, anyhow::Error> {
    let mut item = IncidentWithUsers {
        incident,
        created_by: None,
        acknowledged_by: vec![],
    };
    if let Some(created_by) = &item.incident.created_by {
        let user = user_repository
            .get_user(*created_by, true)
            .await
            .context("failed to get created by user")?;
        if let Some(user) = user {
            item.created_by = Some(user.into());
        }
    }
    for acknowledged_by in &item.incident.acknowledged_by {
        let user = user_repository
            .get_user(*acknowledged_by, true)
            .await
            .context("failed to get acknowledged by user")?;
        if let Some(user) = user {
            item.acknowledged_by.push(user.into());
        }
    }

    Ok(item)
}

/// Enriches a list of incidents with the user information
pub async fn enrich_incidents_with_users(
    incidents: Vec<Incident>,
    user_repository: &impl UserRepository,
) -> Result<Vec<IncidentWithUsers>, anyhow::Error> {
    futures::future::try_join_all(
        incidents
            .into_iter()
            .map(|incident| enrich_incident_with_users(incident, user_repository)),
    )
    .await
}
