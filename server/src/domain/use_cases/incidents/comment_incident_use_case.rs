use chrono::Utc;
use serde::Deserialize;
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        incident_event::{CommentPayload, IncidentEvent, IncidentEventPayload, IncidentEventType},
    },
    ports::{incident_event_repository::IncidentEventRepository, incident_repository::IncidentRepository},
};

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
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn comment_incident<IR: IncidentRepository, IER: IncidentEventRepository<Transaction = IR::Transaction>>(
    auth_context: &AuthContext,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_id: Uuid,
    request: CommentIncidentRequest,
) -> Result<(), CommentIncidentError> {
    if !auth_context.can(Permission::CommentIncidents) {
        return Err(CommentIncidentError::Forbidden);
    }
    let mut tx = incident_event_repo.begin_transaction().await?;
    let incident = incident_repo.get_incident(&mut tx, auth_context.active_organization_id, incident_id).await?;

    if incident.is_none() {
        return Err(CommentIncidentError::IncidentNotFound);
    }

    let event = IncidentEvent {
        organization_id: auth_context.active_organization_id,
        incident_id,
        created_at: Utc::now(),
        event_type: IncidentEventType::Comment,
        event_payload: Some(IncidentEventPayload::Comment(request.payload)),
    };

    incident_event_repo.create_incident_event(&mut tx, event).await?;
    incident_event_repo.commit_transaction(tx).await?;

    Ok(())
}
