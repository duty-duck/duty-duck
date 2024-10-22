use anyhow::Context;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    entities::{incident::IncidentSource, incident_event::{IncidentEvent, IncidentEventType}},
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::IncidentRepository,
    },
};

pub async fn resolve_incidents<IR, IER, INR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    organization_id: Uuid,
    sources: &[IncidentSource],
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    let resolved_incidents = incident_repo
        .resolve_incidents_by_source(transaction, organization_id, sources)
        .await
        .context("Failed to resolve incidents")?;

    for incident_id in resolved_incidents {
        incident_notification_repo
            .cancel_all_notifications_for_incident(transaction, organization_id, incident_id)
            .await
            .context("Failed to cancel pending notifications for incident")?;

        let event = IncidentEvent {
            organization_id,
            incident_id,
            user_id: None,
            created_at: Utc::now(),
            event_type: IncidentEventType::Resolution,
            event_payload: None,
        };

        incident_event_repo.create_incident_event(transaction, event).await.context("Failed to create incident event")?;
    }

    Ok(())
}
