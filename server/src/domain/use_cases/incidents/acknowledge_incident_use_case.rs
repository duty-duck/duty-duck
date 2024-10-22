use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        incident_event::{
            AcknowledgedEventPayload, IncidentEvent, IncidentEventPayload, IncidentEventType,
        },
    },
    ports::{
        incident_event_repository::IncidentEventRepository, incident_notification_repository::IncidentNotificationRepository, incident_repository::IncidentRepository
    },
};

#[derive(Debug, Error)]
pub enum AcknowledgeIncidentError {
    #[error("Incident not found")]
    IncidentNotFound,
    #[error("Current user doesn't have the privilege to acknowledge this incident")]
    Forbidden,
    #[error("Failed to acknowledge incident: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn acknowledge_incident<
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
>(
    auth_context: &AuthContext,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    incident_id: Uuid,
) -> Result<(), AcknowledgeIncidentError> {
    if !auth_context.can(Permission::EditIncidents) {
        return Err(AcknowledgeIncidentError::Forbidden);
    }
    let mut tx = incident_event_repo.begin_transaction().await?;
    let incident = incident_repo
        .get_incident(&mut tx, auth_context.active_organization_id, incident_id)
        .await?;

    match incident {
        Some(incident)
            if incident
                .acknowledged_by
                .contains(&auth_context.active_user_id) =>
        {
            Ok(())
        }
        Some(_) => {
            let event = IncidentEvent {
                organization_id: auth_context.active_organization_id,
                incident_id,
                created_at: Utc::now(),
                user_id: Some(auth_context.active_user_id),
                event_type: IncidentEventType::Acknowledged,
                event_payload: Some(IncidentEventPayload::Acknowledged(
                    AcknowledgedEventPayload {
                        user_id: auth_context.active_user_id,
                    },
                )),
            };

            incident_repo
                .acknowledge_incident(
                    &mut tx,
                    auth_context.active_organization_id,
                    incident_id,
                    auth_context.active_user_id,
                )
                .await?;

            incident_event_repo.create_incident_event(&mut tx, event).await?;
            incident_notification_repo.cancel_all_notifications_for_incident(&mut tx, auth_context.active_organization_id, incident_id).await?;
            incident_event_repo.commit_transaction(tx).await?;
            Ok(())
        }
        None => Err(AcknowledgeIncidentError::IncidentNotFound),
    }
}
