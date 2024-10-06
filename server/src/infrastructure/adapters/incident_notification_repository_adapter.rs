use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{
        entities::incident_notification::IncidentNotification,
        ports::incident_notification_repository::IncidentNotificationRepository,
    },
    postgres_transactional_repo,
};

#[derive(Clone)]
pub struct IncidentNotificationRepositoryAdapter {
    pub pool: PgPool,
}

postgres_transactional_repo!(IncidentNotificationRepositoryAdapter);

#[async_trait::async_trait]
impl IncidentNotificationRepository for IncidentNotificationRepositoryAdapter {
    async fn get_next_notifications_to_send(
        &self,
        tx: &mut Self::Transaction,
        limit: u32,
    ) -> anyhow::Result<Vec<IncidentNotification>> {
        let notifications = sqlx::query!(
            r#"
            DELETE FROM incidents_notifications
            WHERE (organization_id, incident_id, escalation_level) IN (
                SELECT organization_id, incident_id, escalation_level
                FROM incidents_notifications
                WHERE notification_due_at <= NOW()
                ORDER BY notification_due_at
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#,
            limit as i64
        )
        .fetch_all(&mut **tx)
        .await
        .context("Failed to get and delete next notifications to send")?
        .into_iter()
        .map(|record| IncidentNotification {
            organization_id: record.organization_id,
            incident_id: record.incident_id,
            escalation_level: record.escalation_level,
            notification_type: record.notification_type.into(),
            notification_payload: serde_json::from_value(record.notification_payload)
                .expect("Failed to deserialize notification payload"),
            send_sms: record.send_sms,
            send_push_notification: record.send_push_notification,
            send_email: record.send_email,
        })
        .collect();

        Ok(notifications)
    }

    async fn upsert_incident_notification(
        &self,
        tx: &mut Self::Transaction,
        notification: IncidentNotification,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO incidents_notifications (
                organization_id,
                incident_id,
                escalation_level,
                notification_type,
                notification_payload,
                send_sms,
                send_push_notification,
                send_email
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (organization_id, incident_id, escalation_level) DO UPDATE SET
                notification_type = EXCLUDED.notification_type,
                notification_payload = EXCLUDED.notification_payload,
                send_sms = EXCLUDED.send_sms,
                send_push_notification = EXCLUDED.send_push_notification,
                send_email = EXCLUDED.send_email
            "#,
            notification.organization_id,
            notification.incident_id,
            notification.escalation_level,
            notification.notification_type as i16,
            serde_json::to_value(&notification.notification_payload)
                .expect("Failed to serialize notification payload"),
            notification.send_sms,
            notification.send_push_notification,
            notification.send_email
        )
        .execute(&mut **tx)
        .await
        .context("Failed to upsert incident notification")?;

        Ok(())
    }

    async fn cancel_all_notifications_for_incident(
        &self,
        tx: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM incidents_notifications
            WHERE organization_id = $1 AND incident_id = $2
            "#,
            organization_id,
            incident_id
        )
        .execute(&mut **tx)
        .await
        .context("Failed to cancel all notifications for incident")?;

        Ok(())
    }
}
