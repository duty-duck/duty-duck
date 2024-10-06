use crate::domain::{
    entities::{incident::*, incident_event::IncidentEvent},
    ports::incident_repository::{IncidentRepository, ListIncidentsOutput},
};
use async_trait::async_trait;
use itertools::Itertools;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct IncidentRepositoryAdapter {
    pub pool: PgPool,
}

crate::postgres_transactional_repo!(IncidentRepositoryAdapter);

#[async_trait]
impl IncidentRepository for IncidentRepositoryAdapter {
    async fn create_incident(
        &self,
        transaction: &mut Self::Transaction,
        incident: NewIncident,
    ) -> anyhow::Result<Uuid> {
        let cause = match incident.cause {
            Some(cause) => Some(serde_json::to_value(cause)?),
            None => None,
        };
        let (incident_source_type, incident_source_id) = match incident.source {
            IncidentSource::HttpMonitor { id } => (IncidentSourceType::HttpMonitor as i16, id),
        };
        let new_incident_id = sqlx::query!(
            "insert into incidents (
                organization_id,
                created_by,
                status,
                priority,
                cause,
                incident_source_type,
                incident_source_id
            ) 
            values ($1, $2, $3, $4, $5, $6, $7)
            returning id",
            incident.organization_id,
            incident.created_by,
            incident.status as i16,
            incident.priority as i16,
            cause,
            incident_source_type,
            incident_source_id
        )
        .fetch_one(transaction.as_mut())
        .await?
        .id;

        Ok(new_incident_id)
    }

    /// Resolves all incidents for the given sources.
    ///
    /// # Arguments
    ///
    /// * `transaction` - A mutable reference to the transaction object.
    /// * `organization_id` - The ID of the organization to resolve incidents for.
    /// * `sources` - A slice of `IncidentSource` values to resolve incidents for.
    ///
    /// # Returns
    ///
    /// A `Vec<Uuid>` containing the IDs of the resolved incidents.
    async fn resolve_incidents_by_source(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        sources: &[IncidentSource],
    ) -> anyhow::Result<Vec<Uuid>> {
        let http_monitors_ids = sources
            .iter()
            .map(|source| match source {
                IncidentSource::HttpMonitor { id } => *id,
            })
            .unique()
            .collect::<Vec<_>>();

        let resolved_incidents = sqlx::query!(
            "
            UPDATE incidents i
            SET resolved_at = now(), status = $1
            WHERE organization_id = $2
            AND (i.incident_source_type = $3 AND i.incident_source_id = ANY($4::uuid[]))
            RETURNING id
           ",
            &(IncidentStatus::Resolved as i16),
            &organization_id,
            &(IncidentSourceType::HttpMonitor as i16),
            &http_monitors_ids
        )
        .fetch_all(transaction.as_mut())
        .await?
        .into_iter()
        .map(|record| record.id)
        .collect::<Vec<_>>();

        Ok(resolved_incidents)
    }

    async fn list_incidents(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        include_statuses: &[IncidentStatus],
        include_priorities: &[IncidentPriority],
        include_sources: &[IncidentSource],
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<ListIncidentsOutput> {
        let statuses = include_statuses
            .iter()
            .map(|s| *s as i32)
            .collect::<Vec<_>>();
        let priorities = include_priorities
            .iter()
            .map(|s| *s as i32)
            .collect::<Vec<_>>();

        // Used to retrieve incidents by speciifc sources
        let http_monitor_sources_ids = include_sources
            .iter()
            .map(|s| match s {
                IncidentSource::HttpMonitor { id } => *id,
            })
            .collect::<Vec<_>>();

        let total_count = sqlx::query!(
            "SELECT count(DISTINCT id) FROM incidents WHERE organization_id = $1",
            organization_id
        )
        .fetch_one(transaction.as_mut())
        .await?
        .count
        .unwrap_or_default();

        let rows = sqlx::query!(
            "
                SELECT i.*, COUNT(i.id) OVER () as filtered_count from incidents i
                WHERE organization_id = $1 
                AND status IN (SELECT unnest($2::integer[]))
                AND priority IN (SELECT unnest($3::integer[]))
                -- Filter by http monitor ids
                AND (
                   $7::uuid[] = '{}' OR
                   (i.incident_source_type = $6 AND i.incident_source_id = ANY($7::uuid[]))
                )
                ORDER BY created_at DESC
                LIMIT $4 OFFSET $5

            ",
            organization_id,
            &statuses,
            &priorities,
            limit as i64,
            offset as i64,
            &(IncidentSourceType::HttpMonitor as i16),
            &http_monitor_sources_ids
        )
        .fetch_all(transaction.as_mut())
        .await?;

        let total_filtered_count = rows
            .first()
            .and_then(|record| record.filtered_count)
            .unwrap_or_default();

        let incidents = rows
            .into_iter()
            .map(|record| Incident {
                organization_id,
                id: record.id,
                created_at: record.created_at,
                created_by: record.created_by,
                resolved_at: record.resolved_at,
                cause: record
                    .cause
                    .and_then(|value| serde_json::from_value(value).ok()),
                status: record.status.into(),
                priority: record.priority.into(),
                incident_source_id: record.incident_source_id,
                incident_source_type: record.incident_source_type.into(),
            })
            .collect();

        Ok(ListIncidentsOutput {
            total_incidents: total_count as u32,
            total_filtered_incidents: total_filtered_count as u32,
            incidents,
        })
    }

    async fn get_incident(
        &self,
        organization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<Option<Incident>> {
        let record = sqlx::query!(
            "SELECT * FROM incidents WHERE organization_id = $1 AND id = $2",
            organization_id,
            incident_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(record.map(|record| Incident {
            organization_id: record.organization_id,
            id: record.id,
            created_at: record.created_at,
            created_by: record.created_by,
            resolved_at: record.resolved_at,
            cause: record.cause.and_then(|value| serde_json::from_value(value).ok()),
            status: record.status.into(),
            priority: record.priority.into(),
            incident_source_id: record.incident_source_id,
            incident_source_type: record.incident_source_type.into(),
        }))
    }
}
