use anyhow::Context;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

use crate::domain::{entities::authorization::AuthContext, ports::file_storage::{FileStorage, FileStorageKey}};

#[derive(Debug, Error)]
pub enum ServeFileUseCaseError {
    #[error("Failed to list user devices: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn serve_file(
    auth_context: &AuthContext,
    repository: &impl FileStorage,
    file_id: Uuid,
) -> Result<Url, ServeFileUseCaseError> {
    let key = FileStorageKey { organization_id: auth_context.active_organization_id, file_id };
    let url = repository.get_file_url(key).await.context("Failed to get presigned file URL")?;
    Ok(url)
}
