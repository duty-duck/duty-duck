use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::{
    entities::{authorization::{AuthContext, Permission}, incident::Incident}, ports::incident_repository::IncidentRepository,
};

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct GetIncidentResponse {
    pub incident: Incident,
}

#[derive(Error, Debug)]
pub enum GetIncidentError {
    #[error("Incident not found")]
    IncidentNotFound,
    #[error("Current user doesn't have the privilege the see incidents")]
    Forbidden,
    #[error("Failed to get incidents from the database: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn get_incident(
    auth_context: &AuthContext,
    repository: &impl IncidentRepository,
    incident_id: Uuid,
) -> anyhow::Result<GetIncidentResponse, GetIncidentError> {
    if !auth_context.can(Permission::ReadIncidents) {
        return Err(GetIncidentError::Forbidden);
    }

    let mut transaction = repository.begin_transaction().await?;
    match repository.get_incident(&mut transaction, auth_context.active_organization_id, incident_id).await? {
        Some(incident) => Ok(GetIncidentResponse { incident }),
        None => Err(GetIncidentError::IncidentNotFound),
    }
}
