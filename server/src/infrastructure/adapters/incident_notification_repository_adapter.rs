use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{
        entities::incident::{
            Incident, IncidentSourceType, IncidentSourceWithDetails, IncidentWithSourcesDetails,
        },
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
    /// Returns a list of newly-created incidents for which the creation notification has not yet been sent to users
    /// This must be executed inside a transaction. Concurrent transactions will not return the same incidents (incidents that are locked by a transaction will be skipped)
    async fn list_new_incidents_due_for_notification(
        &self,
        tx: &mut Self::Transaction,
        limit: u32,
    ) -> anyhow::Result<Vec<IncidentWithSourcesDetails>> {
        let records = sqlx::query!(
            r#"SELECT
                i.organization_id as "organization_id!",
                i.id as "id!",
                i.created_at as "created_at!",
                i.created_by as "created_by?",
                i.resolved_at as "resolved_at?",
                i.cause as "cause?",
                i.status as "status!",
                i.priority as "priority!",
                i.incident_source_type as "incident_source_type!",
                i.incident_source_id as "incident_source_id!",
                hm.id as "http_monitor_id?",
                hm.url as "http_monitor_url?",
                hm.email_notification_enabled as "http_monitor_email_notification_enabled?",
                hm.push_notification_enabled as "http_monitor_push_notification_enabled?",
                hm.sms_notification_enabled as "http_monitor_sms_notification_enabled?"
            FROM incidents i
            LEFT JOIN http_monitors hm
            ON hm.organization_id = i.organization_id AND hm.id = i.incident_source_id
            WHERE i.id IN (
                SELECT incident_id FROM incidents_notifications
                WHERE creation_notification_sent_at IS NULL
                FOR UPDATE SKIP LOCKED
                LIMIT $1
            )"#,
            limit as i64
        )
        .fetch_all(tx.as_mut())
        .await
        .with_context(|| "Failed to list new incidents due for notfication")?;

        let incidents = records
            .into_iter()
            .map(|record| {
                let incident = Incident {
                    organization_id: record.organization_id,
                    id: record.id,
                    created_at: record.created_at,
                    created_by: record.created_by,
                    resolved_at: record.resolved_at,
                    cause: record
                        .cause
                        .and_then(|value| serde_json::from_value(value).ok()),
                    status: record.status.into(),
                    priority: record.priority.into(),
                    incident_source_type: record.incident_source_type.into(),
                    incident_source_id: record.incident_source_id,
                };

                IncidentWithSourcesDetails {
                    source: match incident.incident_source_type {
                        IncidentSourceType::HttpMonitor => IncidentSourceWithDetails::HttpMonitor {
                            id: incident.incident_source_id,
                            url: record.http_monitor_url.unwrap(),
                            email_notification_enabled: record.http_monitor_email_notification_enabled.unwrap(),
                            push_notification_enabled: record.http_monitor_push_notification_enabled.unwrap(),
                            sms_notification_enabled: record.http_monitor_sms_notification_enabled.unwrap(),
                        },
                    },
                    incident,
                }
            })
            .collect();

        Ok(incidents)
    }

    /// Stores the fact that the creation notification has been sent for an incident
    async fn acknowledge_incident_creation_notification(
        &self,
        tx: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE incidents_notifications SET creation_notification_sent_at = NOW() WHERE organization_id = $1 AND incident_id = $2",
            organization_id,
            incident_id
        )
        .execute(tx.as_mut())
        .await
        .with_context(|| "Failed to list new incidents due for notfication")?;
        Ok(())
    }
}
