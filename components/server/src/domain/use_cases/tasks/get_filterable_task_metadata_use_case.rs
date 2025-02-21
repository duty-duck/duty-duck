use crate::domain::{
    entities::{authorization::AuthContext, entity_metadata::FilterableMetadata},
    ports::task_repository::TaskRepository,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetFilterableTaskMetadataError {
    #[error("Failed to get filterable metadata from the database: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn get_filterable_task_metadata<IR: TaskRepository>(
    auth_context: &AuthContext,
    task_repo: &IR,
) -> Result<FilterableMetadata, GetFilterableTaskMetadataError> {
    task_repo
        .get_filterable_metadata(auth_context.active_organization_id)
        .await
        .map_err(GetFilterableTaskMetadataError::TechnicalFailure)
}
