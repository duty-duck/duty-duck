use anyhow::Context;
use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        incident::{Incident, IncidentStatus},
        incident_event::{IncidentEvent, IncidentEventType},
    },
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::IncidentRepository,
    },
};

#[derive(Debug, Error)]
pub enum ResolveIncidentError {
    #[error("Incident not found")]
    IncidentNotFound,
    #[error("Incident already resolved")]
    IncidentAlreadyResolved,
    #[error("Current user doesn't have the privilege the resolve incidents")]
    Forbidden,
    #[error("Failed to resolve event: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveIncidentOutput {
    IncidentResolved,
    IncidentDeleted,
}

pub async fn resolve_incident_manually<IR, IER, INR>(
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    auth_context: &AuthContext,
    incident_id: Uuid,
) -> Result<(), ResolveIncidentError>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    if !auth_context.can(Permission::EditIncidents) {
        return Err(ResolveIncidentError::Forbidden);
    }

    let mut tx = incident_repo.begin_transaction().await?;
    let incident = match incident_repo
        .get_incident(&mut tx, auth_context.active_organization_id, incident_id)
        .await?
    {
        Some(incident) => incident,
        None => return Err(ResolveIncidentError::IncidentNotFound),
    };

    resolve_incident(
        &mut tx,
        incident_repo,
        incident_event_repo,
        incident_notification_repo,
        &incident,
        Some(auth_context.active_user_id),
    )
    .await?;

    incident_repo.commit_transaction(tx).await?;

    Ok(())
}

/// Resolves an incident and sends the appropriate notifications
/// If the incident is already resolved, it returns an error
/// If the incident is to be confirmed, it deletes the incident without sending any notifications
/// This function is used in several other use cases where incidents are resolved automatically, and also as the foundation of the "resolve_incident_manually" use case
pub async fn resolve_incident<IR, IER, INR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    incident: &Incident,
    resolved_by_user: Option<Uuid>,
) -> Result<ResolveIncidentOutput, ResolveIncidentError>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    match incident.status {
        IncidentStatus::Resolved => Err(ResolveIncidentError::IncidentAlreadyResolved),
        IncidentStatus::ToBeConfirmed => {
            incident_notification_repo
                .cancel_all_notifications_for_incident(
                    transaction,
                    incident.organization_id,
                    incident.id,
                )
                .await
                .context("Failed to cancel pending notifications for incident")?;

            incident_repo
                .delete_incident(transaction, incident.organization_id, incident.id)
                .await
                .context("Failed to delete incident")?;

            Ok(ResolveIncidentOutput::IncidentDeleted)
        }
        IncidentStatus::Ongoing => {
            incident_repo
                .update_incident(
                    transaction,
                    Incident {
                        status: IncidentStatus::Resolved,
                        resolved_at: Some(Utc::now()),
                        ..incident.clone()
                    },
                )
                .await
                .context("Failed to resolve incident")?;

            incident_notification_repo
                .cancel_all_notifications_for_incident(
                    transaction,
                    incident.organization_id,
                    incident.id,
                )
                .await
                .context("Failed to cancel pending notifications for incident")?;

            let event = IncidentEvent {
                organization_id: incident.organization_id,
                incident_id: incident.id,
                user_id: resolved_by_user,
                created_at: Utc::now(),
                event_type: IncidentEventType::Resolution,
                event_payload: None,
            };

            incident_event_repo
                .create_incident_event(transaction, event)
                .await
                .context("Failed to create incident event")?;

            Ok(ResolveIncidentOutput::IncidentResolved)
        }
    }
}
