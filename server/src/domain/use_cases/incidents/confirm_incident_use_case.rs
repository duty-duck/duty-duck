use anyhow::Context;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    entities::{incident::IncidentSource, incident_event::{IncidentEvent, IncidentEventType}, incident_notification::{IncidentNotification, IncidentNotificationType}},
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::IncidentRepository,
    },
};

use super::NotificationOpts;

/// Confirms all the ongoing incidents for the given sources and sends the appropriate notifications
pub async fn confirm_incidents<IR, IER, INR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    organization_id: Uuid,
    sources: &[IncidentSource],
    notification_opts: Option<NotificationOpts>,
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    let confirmed_incidents = incident_repo
        .confirm_incidents_by_source(transaction, organization_id, sources)
        .await
        .context("Failed to confirm incidents")?;

    for incident_id in confirmed_incidents {
        let event = IncidentEvent {
            organization_id,
            incident_id,
            user_id: None,
            created_at: Utc::now(),
            event_type: IncidentEventType::Confirmation,
            event_payload: None,
        };

        incident_event_repo.create_incident_event(transaction, event).await.context("Failed to create incident event")?;

        if let Some(NotificationOpts {
            send_sms,
            send_push_notification,
            send_email,
            notification_payload,
        }) = notification_opts.clone()
        {
            let event_notification = IncidentNotification {
                incident_id,
                organization_id,
                escalation_level: 0,
                notification_type: IncidentNotificationType::IncidentConfirmation,
                notification_due_at: Utc::now(),
                notification_payload,
                send_sms,
                send_push_notification,
                send_email,
            };
    
            incident_notification_repo
                .upsert_incident_notification(transaction, event_notification)
                .await?;
        }
    }

    Ok(())
}
