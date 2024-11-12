use anyhow::Context;
use chrono::Utc;

use crate::domain::{
    entities::{
        incident::{Incident, IncidentStatus},
        incident_event::{IncidentEvent, IncidentEventType},
    },
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::IncidentRepository,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveIncidentOutput {
    IncidentResolved,
    IncidentDeleted,
}

/// Resolves an incident and sends the appropriate notifications
/// If the incident is already resolved, it returns an error
/// If the incident is to be confirmed, it deletes the incident without sending any notifications
pub async fn resolve_incident<IR, IER, INR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    incident: &Incident,
) -> anyhow::Result<ResolveIncidentOutput>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    match incident.status {
        IncidentStatus::Resolved => Err(anyhow::anyhow!("Incident is already resolved")),
        IncidentStatus::ToBeConfirmed => {
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
            .cancel_all_notifications_for_incident(transaction, incident.organization_id, incident.id)
            .await
            .context("Failed to cancel pending notifications for incident")?;
    
        let event = IncidentEvent {
            organization_id: incident.organization_id,
            incident_id: incident.id,
            user_id: None,
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
