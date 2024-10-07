use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        incident::{IncidentPriority, IncidentSource, IncidentStatus},
    },
    ports::incident_repository::{IncidentRepository, ListIncidentsOutput},
    use_cases::incidents::{ListIncidentsError, ListIncidentsParams, ListIncidentsResponse},
};

pub async fn list_http_monitor_incidents(
    auth_context: &AuthContext,
    repository: &impl IncidentRepository,
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
            &[IncidentSource::HttpMonitor { id: monitor_id }],
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
