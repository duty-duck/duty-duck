use crate::domain::{
    entities::incident::*,
    ports::incident_repository::{IncidentRepository, ListIncidentsOutput},
};
use async_trait::async_trait;
use chrono::*;
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

    /// Lists all incidents for the given organization.
    ///
    /// # Arguments
    ///
    /// * `transaction` - A mutable reference to the transaction object.
    /// * `organization_id` - The ID of the organization to list incidents for.
    /// * `include_statuses` - A slice of `IncidentStatus` values to include in the results. Make sure to include every status you are interested in. Otherwise, the query will return an empty list.
    /// * `include_priorities` - A slice of `IncidentPriority` values to include in the results. Make sure to include every priority you are interested in. Otherwise, the query will return an empty list.
    /// * `include_sources` - A slice of `IncidentSource` values to include in the results.
    /// * `limit` - The maximum number of incidents to return.
    /// * `offset` - The number of incidents to skip before returning the results.
    /// * `from_date` - The start date to filter incidents by.
    /// * `to_date` - The end date to filter incidents by.
    ///
    /// # Returns
    ///
    /// A `ListIncidentsOutput` struct containing the incidents, total number of incidents, and total number of filtered incidents.
    #[allow(clippy::too_many_arguments)]
    async fn list_incidents(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        include_statuses: &[IncidentStatus],
        include_priorities: &[IncidentPriority],
        include_sources: &[IncidentSource],
        limit: u32,
        offset: u32,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
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
                -- Filter by date
                AND ($8::timestamptz IS NULL OR i.created_at >= $8::timestamptz)
                AND ($9::timestamptz IS NULL OR i.created_at <= $9::timestamptz)
                ORDER BY created_at DESC
                LIMIT $4 OFFSET $5
            ",
            organization_id,
            &statuses,
            &priorities,
            limit as i64,
            offset as i64,
            &(IncidentSourceType::HttpMonitor as i16),
            &http_monitor_sources_ids,
            from_date,
            to_date
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
                acknowledged_by: record.acknowledged_by,
            })
            .collect();

        Ok(ListIncidentsOutput {
            total_incidents: total_count as u32,
            total_filtered_incidents: total_filtered_count as u32,
            incidents,
        })
    }

    /// Gets the incident with the given ID.
    ///
    /// # Arguments
    ///
    /// * `organization_id` - The ID of the organization to get the incident for.
    /// * `incident_id` - The ID of the incident to get.
    /// * `transaction` - A mutable reference to the transaction object.
    ///
    /// # Returns
    ///
    /// An `Option<Incident>` containing the incident if it exists, or `None` if it does not.
    async fn get_incident(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<Option<Incident>> {
        let record = sqlx::query!(
            "SELECT * FROM incidents WHERE organization_id = $1 AND id = $2",
            organization_id,
            incident_id
        )
        .fetch_optional(transaction.as_mut())
        .await?;
        Ok(record.map(|record| Incident {
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
            incident_source_id: record.incident_source_id,
            incident_source_type: record.incident_source_type.into(),
            acknowledged_by: record.acknowledged_by,
        }))
    }

    /// Marks the incident as acknowledged by the given user.
    ///
    /// # Arguments
    ///
    /// * `transaction` - A mutable reference to the transaction object.
    /// * `organization_id` - The ID of the organization to acknowledge incidents for.
    /// * `incident_id` - The ID of the incident to acknowledge.
    /// * `user_id` - The ID of the user acknowledging the incident.
    async fn acknowledge_incident(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<()> {
        sqlx::query!("UPDATE incidents SET acknowledged_by = array_append(acknowledged_by, $1) WHERE organization_id = $2 AND id = $3", user_id, organization_id, incident_id)
            .execute(transaction.as_mut())
            .await?;
        Ok(())
    }
}
