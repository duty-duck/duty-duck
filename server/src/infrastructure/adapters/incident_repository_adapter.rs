use std::collections::HashSet;

use crate::domain::{
    entities::incident::{
        Incident, IncidentPriority, IncidentSource, IncidentStatus, IncidentWithSources,
    },
    ports::{
        incident_repository::{IncidentRepository, ListIncidentsOutput, NewIncident},
        transactional_repository::TransactionalRepository,
    },
};
use anyhow::Context;
use async_trait::async_trait;
use bigdecimal::ToPrimitive;
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
        let new_incident_id = sqlx::query!(
            "insert into incidents (
                organization_id,
                created_by,
                status,
                priority
            ) 
            values ($1, $2, $3, $4)
            returning id",
            incident.organization_id,
            incident.created_by,
            incident.status as i16,
            incident.priority as i16,
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
            UPDATE incidents
            SET resolved_at = now(), status = $1
            FROM http_monitors_incidents hmi
            WHERE incidents.organization_id = $2 and incidents.id = hmi.incident_id and hmi.organization_id = $2 and hmi.http_monitor_id IN (SELECT unnest($3::uuid[]))",
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
        organization_id: Uuid,
        include_statuses: Vec<IncidentStatus>,
        include_priorities: Vec<IncidentPriority>,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<ListIncidentsOutput> {
        let mut tx = self.begin_transaction().await?;
        let statuses = include_statuses
            .into_iter()
            .map(|s| s as i32)
            .collect::<Vec<_>>();
        let priorities = include_priorities
            .into_iter()
            .map(|s| s as i32)
            .collect::<Vec<_>>();

        // Sadly there is no way to type-check this query using the query_as! macro at the moment, becasue the macro ignores the #[sqlx(flatten)] attribute.
        // See https://github.com/launchbadge/sqlx/issues/514 and https://github.com/launchbadge/sqlx/issues/1121
        let incidents: Vec<IncidentWithSourceRow> = sqlx::query_as(
            "SELECT i.*, hmi.http_monitor_id 
                FROM (
                    SELECT * FROM incidents WHERE organization_id = $1 AND status IN (SELECT unnest($2::integer[])) AND priority IN (SELECT unnest($3::integer[])) 
                    ORDER BY created_at DESC 
                    LIMIT $4 
                    OFFSET $5
                ) as i
                LEFT JOIN http_monitors_incidents hmi ON hmi.organization_id = i.organization_id AND hmi.incident_id = i.id
            "
        ).bind(organization_id).bind(&statuses).bind(&priorities).bind(limit as i64).bind(offset as i64)
        .fetch_all(&mut *tx)
        .await?;

        let total_count = sqlx::query!(
            "SELECT count(*) FROM incidents WHERE organization_id = $1",
            organization_id
        )
        .fetch_one(&mut *tx)
        .await?
        .count
        .unwrap_or_default();

        let count_total_res = sqlx::query!(
            "SELECT count(*) as count, SUM(extract(epoch from ( coalesce(resolved_at, now()) - created_at ))) as duration
             FROM incidents 
             WHERE organization_id = $1 AND status IN (SELECT unnest($2::integer[])) AND priority IN (SELECT unnest($3::integer[]))",
            organization_id,
            &statuses,
            &priorities
        )
        .fetch_one(&mut *tx)
        .await?;

        let total_filtered_count = count_total_res.count.unwrap_or_default();
        let sum_filtered_incidents_duration = count_total_res
            .duration
            .unwrap_or_default()
            .to_u32()
            .with_context(|| "Cannot converted summed duration to u32")?;

        tx.commit().await?;

        // Group consecutive rows wit the same incident id
        let chunks = incidents.into_iter().chunk_by(|row| row.incident.id);

        // Transform each chunk into a single incident with all its sources
        let incidents = chunks
            .into_iter()
            .filter_map(|(_, chunk)| {
                let mut incident = None;
                let mut sources = HashSet::new();
                for row in chunk {
                    if incident.is_none() {
                        incident = Some(row.incident)
                    }
                    if let Some(id) = row.http_monitor_id {
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
            sum_filtered_incidents_duration,
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

#[derive(sqlx::FromRow)]
struct IncidentWithSourceRow {
    #[sqlx(flatten)]
    incident: Incident,
    http_monitor_id: Option<Uuid>,
}
