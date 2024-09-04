use std::collections::HashSet;

use anyhow::Context;
use itertools::Itertools;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{
        entities::incident::{Incident, IncidentSourceWithDetails, IncidentWithSourcesDetails},
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
    /// Returns a list of newly-created incidents for which the creation notification has not yet been sent to userrs
    /// This must be executed inside a transaction. Concurrent transactions will not return the same incidents (incidents that are locked by a transaction will be skipped)
    async fn list_new_incidents_due_for_notification(
        &self,
        tx: &mut Self::Transaction,
        limit: u32,
    ) -> anyhow::Result<Vec<IncidentWithSourcesDetails>> {
        let records = sqlx::query!(
            "SELECT
                i.*, 
                hmi.http_monitor_id,
                hm.url as http_monitor_url
            FROM incidents i
            LEFT JOIN http_monitors_incidents hmi
            ON hmi.organization_id = i.organization_id AND hmi.incident_id = i.id
            LEFT JOIN http_monitors hm
            ON hm.organization_id = i.organization_id AND hm.id = hmi.http_monitor_id
            WHERE i.id IN (
                SELECT incident_id FROM incidents_notifications
                WHERE creation_notification_sent_at IS NULL
                FOR UPDATE SKIP LOCKED
                LIMIT $1
            )",
            limit as i64
        )
        .fetch_all(tx.as_mut())
        .await
        .with_context(|| "Failed to list new incidents due for notfication")?;

        // Group consecutive rows wit the same incident id
        let incidents = records
            .into_iter()
            .chunk_by(|record| record.id)
            .into_iter()
            .filter_map(|(_, chunk)| {
                let mut incident = None;
                let mut sources = HashSet::new();

                for record in chunk {
                    if incident.is_none() {
                        incident = Some(Incident {
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
                        })
                    }

                    if let Some(id) = record.http_monitor_id {
                        sources.insert(IncidentSourceWithDetails::HttpMonitor {
                            id,
                            url: record.http_monitor_url.unwrap_or_default(),
                        });
                    }
                }

                Some(IncidentWithSourcesDetails {
                    incident: incident?,
                    sources,
                })
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
