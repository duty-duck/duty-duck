use std::collections::HashSet;

use crate::domain::{
    entities::incident::{
        self, Incident, IncidentPriority, IncidentSource, IncidentStatus, IncidentWithSources,
    },
    ports::{
        incident_repository::{IncidentRepository, ListIncidentsOutput, NewIncident},
        transactional_repository::TransactionalRepository,
    },
};
use async_trait::async_trait;
use itertools::Itertools;
use sqlx::{FromRow, PgPool};
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
        let new_incident_id = sqlx::query!(
            "insert into incidents (
                organization_id,
                created_by,
                status,
                priority,
                cause
            ) 
            values ($1, $2, $3, $4, $5)
            returning id",
            incident.organization_id,
            incident.created_by,
            incident.status as i16,
            incident.priority as i16,
            cause
        )
        .fetch_one(transaction.as_mut())
        .await?
        .id;

        for source in incident.sources {
            self.attach_incident_to_source(
                transaction,
                incident.organization_id,
                new_incident_id,
                source,
            )
            .await?;
        }

        Ok(new_incident_id)
    }

    async fn resolve_incidents_by_source(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        sources: &[IncidentSource],
    ) -> anyhow::Result<()> {
        let http_monitors_ids = sources
            .iter()
            .map(|source| match source {
                IncidentSource::HttpMonitor { id } => *id,
            })
            .unique()
            .collect::<Vec<_>>();

        sqlx::query!("
            UPDATE incidents i
            SET resolved_at = now(), status = $1
            WHERE organization_id = $2
            AND ($3::uuid[] = '{}' OR EXISTS (
                    SELECT 1 FROM http_monitors_incidents hmi 
                    WHERE hmi.organization_id = i.organization_id 
                    AND hmi.incident_id = i.id
                    AND hmi.http_monitor_id IN (SELECT UNNEST ($3::uuid[])))
            )",
            &(IncidentStatus::Resolved as i16),
            &organization_id,
            &http_monitors_ids
        )
        .execute(transaction.as_mut())
        .await?;

        Ok(())
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

        let incidents = sqlx::query!(
            "
            -- First, in a subquery, get the matching incidents and paginate
            WITH incidents AS (
                SELECT i.*, COUNT(i.id) OVER () as filtered_count from incidents i
                WHERE organization_id = $1 
                AND status IN (SELECT unnest($2::integer[]))
                AND priority IN (SELECT unnest($3::integer[]))
                AND ($6::uuid[] = '{}' OR EXISTS (
                    SELECT 1 FROM http_monitors_incidents hmi 
                    WHERE hmi.organization_id = i.organization_id 
                    AND hmi.incident_id = i.id
                    AND hmi.http_monitor_id IN (SELECT UNNEST ($6::uuid[])))
                )
                ORDER BY created_at DESC
                LIMIT $4 OFFSET $5
            )
            SELECT i.*, hmi.http_monitor_id
            FROM incidents i
            -- then join with the sources
            LEFT JOIN http_monitors_incidents hmi
                ON hmi.organization_id = $1 AND hmi.incident_id = i.id
            ",
            organization_id,
            &statuses,
            &priorities,
            limit as i64,
            offset as i64,
            &http_monitor_sources_ids
        )
        .fetch_all(transaction.as_mut())
        .await?;

        let total_filtered_count = incidents
            .first()
            .and_then(|record| record.filtered_count)
            .unwrap_or_default();

        // Group consecutive rows wit the same incident id
        let incidents = incidents
            .into_iter()
            .chunk_by(|record| record.id)
            .into_iter()
            .filter_map(|(_, chunk)| {
                let mut incident = None;
                let mut sources = HashSet::new();

                for record in chunk {
                    if incident.is_none() {
                        incident = Some(Incident {
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
                        })
                    }

                    if let Some(id) = record.http_monitor_id {
                        sources.insert(IncidentSource::HttpMonitor { id });
                    }
                }

                Some(IncidentWithSources {
                    incident: incident?,
                    sources,
                })
            })
            .collect();

        Ok(ListIncidentsOutput {
            total_incidents: total_count as u32,
            total_filtered_incidents: total_filtered_count as u32,
            incidents,
        })
    }
}

impl IncidentRepositoryAdapter {
    async fn attach_incident_to_source(
        &self,
        transaction: &mut <Self as TransactionalRepository>::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
        source: IncidentSource,
    ) -> anyhow::Result<()> {
        match source {
            IncidentSource::HttpMonitor { id } => {
                sqlx::query!("insert into http_monitors_incidents (organization_id, incident_id, http_monitor_id) values ($1, $2, $3)", organization_id, incident_id, id)
                    .execute(transaction.as_mut())
                    .await?;
            }
        }
        Ok(())
    }
}
