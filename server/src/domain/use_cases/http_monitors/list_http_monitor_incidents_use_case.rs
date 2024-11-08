use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        incident::{IncidentPriority, IncidentSource, IncidentStatus},
    },
    ports::{
        incident_repository::{IncidentRepository, ListIncidentsOpts, ListIncidentsOutput},
        user_repository::UserRepository,
    },
    use_cases::{
        incidents::{
            enrich_incidents_with_users, ListIncidentsError, ListIncidentsParams, ListIncidentsResponse, OrderIncidentsBy
        },
        shared::OrderDirection,
    },
};

pub async fn list_http_monitor_incidents(
    auth_context: &AuthContext,
    incident_repository: &impl IncidentRepository,
    user_repository: &impl UserRepository,
    monitor_id: Uuid,
    params: ListIncidentsParams,
) -> Result<ListIncidentsResponse, ListIncidentsError> {
    if !auth_context.can(Permission::ReadIncidents) {
        return Err(ListIncidentsError::Forbidden);
    }

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
                include_sources: &[IncidentSource::HttpMonitor { id: monitor_id }],
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
        items: enrich_incidents_with_users(incidents, user_repository).await?,
        total_number_of_filtered_results: total_filtered_incidents,
        total_number_of_results: total_incidents,
    })
}
