use crate::domain::{
    entities::{authorization::AuthContext, entity_metadata::FilterableMetadata},
    ports::http_monitor_repository::HttpMonitorRepository,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetFilterableHttpMonitorMetadataError {
    #[error("Failed to get filterable metadata from the database: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn get_filterable_http_monitor_metadata<IR: HttpMonitorRepository>(
    auth_context: &AuthContext,
    http_monitor_repo: &IR,
) -> Result<FilterableMetadata, GetFilterableHttpMonitorMetadataError> {
    http_monitor_repo
        .get_filterable_metadata(auth_context.active_organization_id)
        .await
        .map_err(GetFilterableHttpMonitorMetadataError::TechnicalFailure)
}
