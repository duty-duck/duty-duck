use std::sync::Arc;

use anyhow::Context;
use sqlx::postgres::PgPool;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::domain::{
    entities::incident_event::IncidentEvent,
    ports::incident_event_repository::IncidentEventRepository,
};

#[derive(Clone)]
pub struct IncidentEventRepositoryAdapter {
    pool: PgPool,
    _partition_creation_background_task: Arc<JoinHandle<()>>,
}

impl IncidentEventRepositoryAdapter {
    pub async fn new(pool: PgPool) -> Self {
        let partitions_created = Arc::new(tokio::sync::Notify::new());

        let partition_creation_background_task = tokio::spawn({
            let partitions_created = partitions_created.clone();
            let pool = pool.clone();
            async move {
                let mut interval =
                    tokio::time::interval(std::time::Duration::from_secs(60 * 60 * 24));
                loop {
                    interval.tick().await;
                    match sqlx::query!("SELECT create_incident_timeline_partition_for_month()")
                        .execute(&pool)
                        .await
                    {
                        Ok(_) => tracing::debug!("Incident timeline partition created"),
                        Err(e) => {
                            tracing::error!("Error creating incident timeline partition: {:?}", e)
                        }
                    }
                    partitions_created.notify_waiters();
                }
            }
        });

        // wait for the first partition to be created before returning the adapter
        partitions_created.notified().await;

        Self {
            pool,
            _partition_creation_background_task: Arc::new(partition_creation_background_task),
        }
    }
}

crate::postgres_transactional_repo!(IncidentEventRepositoryAdapter);

#[async_trait::async_trait]
impl IncidentEventRepository for IncidentEventRepositoryAdapter {
    async fn create_incident_event(
        &self,
        tx: &mut Self::Transaction,
        event: IncidentEvent,
    ) -> anyhow::Result<()> {
        let event_created_at = event.created_at;
        let event_type = event.event_type;

        sqlx::query!(
            "INSERT INTO incident_timeline_events (organization_id, incident_id, user_id, created_at, event_type, event_payload)
            VALUES ($1, $2, $3, $4, $5, $6)",
            event.organization_id,
            event.incident_id,
            event.user_id,
            event.created_at,
            event.event_type as i16,
            serde_json::to_value(event.event_payload)?
        )
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to persist incident event with type {:?} and timestamp: {}",
                event.event_type, event_created_at
            )
        })?;

        Ok(())
    }

    async fn get_incident_timeline(
        &self,
        organization_id: Uuid,
        incident_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<Vec<IncidentEvent>> {
        let events = sqlx::query!(
            "SELECT incident_timeline_events.* FROM incidents, incident_timeline_events
            WHERE incidents.organization_id = $1 AND incidents.id = $2 AND incidents.id = incident_timeline_events.incident_id
            -- this should help postgres select the corrent partition for the events
            AND incident_timeline_events.created_at >= incidents.created_at
            ORDER BY created_at ASC
            LIMIT $3 OFFSET $4",
            organization_id,
            incident_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch incident timeline events")?
        .into_iter()
        .map(|record| IncidentEvent {
            organization_id: record.organization_id,
            incident_id: record.incident_id,
            created_at: record.created_at,
            user_id: record.user_id,
            event_type: record.event_type.into(),
            event_payload: record
                .event_payload
                .and_then(|payload| serde_json::from_value(payload).ok()),
        })
        .collect();

        Ok(events)
    }
}
