use anyhow::Context;
use chrono::Utc;

use crate::domain::{
    entities::{
        incident::{Incident, IncidentStatus},
        incident_event::{IncidentEvent, IncidentEventType},
        incident_notification::{IncidentNotification, IncidentNotificationType},
    },
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::IncidentRepository,
    },
};

#[cfg(test)]
mod tests;

use super::NotificationOpts;

/// Confirms a to-be-confirmed incident and sends the appropriate notifications
pub async fn confirm_incident<IR, IER, INR>(
    transaction: &mut IR::Transaction,
    incident_repo: &IR,
    incident_event_repo: &IER,
    incident_notification_repo: &INR,
    incident: &Incident,
    NotificationOpts {
        send_sms,
        send_push_notification,
        send_email,
        notification_payload,
    }: NotificationOpts,
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    if incident.status != IncidentStatus::ToBeConfirmed {
        return Err(anyhow::anyhow!("Incident is not to be confirmed"));
    }

    incident_repo
        .update_incident(
            transaction,
            Incident {
                status: IncidentStatus::Ongoing,
                ..incident.clone()
            },
        )
        .await
        .context("Failed to update incident")?;

    let event = IncidentEvent {
        organization_id: incident.organization_id,
        incident_id: incident.id,
        user_id: None,
        created_at: Utc::now(),
        event_type: IncidentEventType::Confirmation,
        event_payload: None,
    };

    incident_event_repo
        .create_incident_event(transaction, event)
        .await
        .context("Failed to create incident event")?;

    let event_notification = IncidentNotification {
        incident_id: incident.id,
        organization_id: incident.organization_id,
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
    Ok(())
}
