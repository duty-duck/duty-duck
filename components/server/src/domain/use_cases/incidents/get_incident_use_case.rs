use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        incident::IncidentWithUsers,
    },
    ports::{incident_repository::IncidentRepository, user_repository::UserRepository},
};

use super::enrich_incident_with_users;

#[derive(Debug, Serialize, TS, ToSchema)]
#[ts(export)]
pub struct GetIncidentResponse {
    pub incident: IncidentWithUsers,
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
    incident_repository: &impl IncidentRepository,
    user_repository: &impl UserRepository,
    incident_id: Uuid,
) -> anyhow::Result<GetIncidentResponse, GetIncidentError> {
    if !auth_context.can(Permission::ReadIncidents) {
        return Err(GetIncidentError::Forbidden);
    }

    let mut transaction = incident_repository.begin_transaction().await?;
    match incident_repository
        .get_incident(
            &mut transaction,
            auth_context.active_organization_id,
            incident_id,
        )
        .await?
    {
        Some(incident) => Ok(GetIncidentResponse {
            incident: enrich_incident_with_users(incident, user_repository).await?,
        }),
        None => Err(GetIncidentError::IncidentNotFound),
    }
}
