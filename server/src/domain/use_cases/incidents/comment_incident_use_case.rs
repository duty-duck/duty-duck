use serde::Deserialize;
use thiserror::Error;
use ts_rs::TS;

use crate::domain::entities::incident_event::CommentPayload;

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CommentIncidentRequest {
    payload: CommentPayload,
}

#[derive(Debug, Error)]
pub enum CommentIncidentError {
    #[error("Incident not found")]
    IncidentNotFound,
    #[error("Current user doesn't have the privilege to comment this incident")]
    Forbidden,
    #[error("Failed to comment incident: {0}")]
    TechnicalFailure(anyhow::Error),
}
