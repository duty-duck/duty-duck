use crate::domain::{
    entities::{authorization::AuthContext, entity_metadata::FilterableMetadata},
    ports::incident_repository::IncidentRepository,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetFilterableIncidentMetadataError {
    #[error("Failed to get filterable metadata from the database: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn get_filterable_incident_metadata<IR: IncidentRepository>(
    auth_context: &AuthContext,
    incident_repo: &IR,
) -> Result<FilterableMetadata, GetFilterableIncidentMetadataError> {
    incident_repo
        .get_filterable_metadata(auth_context.active_organization_id)
        .await
        .map_err(GetFilterableIncidentMetadataError::TechnicalFailure)
}
