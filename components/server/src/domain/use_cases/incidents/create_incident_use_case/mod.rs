use anyhow::Context;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    entities::{
        incident::NewIncident,
        incident_event::{IncidentEvent, IncidentEventType},
        incident_notification::{
            IncidentNotification, IncidentNotificationPayload, IncidentNotificationType,
        },
    },
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::IncidentRepository,
    },
};

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct NotificationOpts {
    pub send_sms: bool,
    pub send_push_notification: bool,
    pub send_email: bool,
    pub notification_payload: IncidentNotificationPayload,
}

/// Creates an incident and the associated incident event and notification
pub async fn create_incident<IR, IER, INR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    new_incident: NewIncident,
    notification_opts: Option<NotificationOpts>,
) -> anyhow::Result<Uuid>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    let incident_id = incident_repo
        .create_incident(transaction, new_incident.clone())
        .await
        .context("Failed to persist incident")?;

    let event = IncidentEvent {
        incident_id,
        organization_id: new_incident.organization_id,
        user_id: None,
        created_at: Utc::now(),
        event_type: IncidentEventType::Creation,
        event_payload: None,
    };

    incident_event_repo
        .create_incident_event(transaction, event)
        .await?;

    if let Some(NotificationOpts {
        send_sms,
        send_push_notification,
        send_email,
        notification_payload,
    }) = notification_opts
    {
        let event_notification = IncidentNotification {
            incident_id,
            organization_id: new_incident.organization_id,
            escalation_level: 0,
            notification_type: IncidentNotificationType::IncidentCreation,
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

    Ok(incident_id)
}
