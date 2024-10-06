use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::domain::{entities::incident_event::IncidentEvent, ports::incident_event_repository::IncidentEventRepository};

#[derive(Clone)]
pub struct IncidentEventRepositoryAdapter {
    pub pool: PgPool,
}

impl IncidentEventRepositoryAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
        sqlx::query!(
            "INSERT INTO incident_timeline_events (organization_id, incident_id, created_at, event_type, event_payload)
            VALUES ($1, $2, $3, $4, $5)",
            event.organization_id,
            event.incident_id,
            event.created_at,
            event.event_type as i16,
            serde_json::to_value(event.event_payload)?
        )
        .execute(&mut **tx)
        .await?;

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
            "SELECT * FROM incident_timeline_events WHERE organization_id = $1 AND incident_id = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4",
            organization_id,
            incident_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|record| IncidentEvent {
            organization_id: record.organization_id,
            incident_id: record.incident_id,
            created_at: record.created_at,
            event_type: record.event_type.into(),
            event_payload: record
                .event_payload
                .and_then(|payload| serde_json::from_value(payload).ok()),
        })
        .collect();

        Ok(events)
    }
}
