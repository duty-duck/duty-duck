use crate::domain::{
    entities::incident::*,
    ports::incident_repository::{IncidentRepository, ListIncidentsOpts, ListIncidentsOutput},
    use_cases::{incidents::OrderIncidentsBy, shared::OrderDirection},
};
use async_trait::async_trait;
use itertools::Itertools;
use sqlx::{PgPool, Row};
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
                metadata,
                cause,
                incident_source_type,
                incident_source_id
            ) 
            values ($1, $2, $3, $4, $5, $6, $7, $8)
            returning id",
            incident.organization_id,
            incident.created_by,
            incident.status as i16,
            incident.priority as i16,
            serde_json::to_value(incident.metadata)?,
            cause,
            incident_source_type,
            incident_source_id
        )
        .fetch_one(transaction.as_mut())
        .await?
        .id;

        Ok(new_incident_id)
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
    async fn list_incidents<'a>(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        opts: ListIncidentsOpts<'a>,
    ) -> anyhow::Result<ListIncidentsOutput> {
        let statuses = opts
            .include_statuses
            .iter()
            .map(|s| *s as i32)
            .collect::<Vec<_>>();
        let priorities = opts
            .include_priorities
            .iter()
            .map(|s| *s as i32)
            .collect::<Vec<_>>();

        // Used to retrieve incidents by speciifc sources
        let http_monitor_sources_ids = opts
            .include_sources
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

        // here we must use a dynamic sql query instead of the `sqlx::query!` macro because
        // we need to order results by a dynamic column name
        let rows = sqlx::query(&format!(
            "
                SELECT i.*, COUNT(i.id) OVER () as filtered_count from incidents i
                WHERE organization_id = $1 
                -- Filter by status
                AND status IN (SELECT unnest($2::integer[]))

                -- Filter by priority
                AND priority IN (SELECT unnest($3::integer[]))

                -- Filter by http monitor ids
                AND (
                   $7::uuid[] = '{{}}' OR
                   (i.incident_source_type = $6 AND i.incident_source_id = ANY($7::uuid[]))
                )

                -- Filter by date (ongoing incidents are always returned)
                AND (
                    i.status = 1 OR
                    ($8::timestamptz IS NULL OR i.created_at >= $8::timestamptz) AND ($9::timestamptz IS NULL OR i.created_at <= $9::timestamptz)
                )
                ORDER BY {} {}
                LIMIT $4 OFFSET $5
            ",
            match opts.order_by {
                OrderIncidentsBy::CreatedAt => "created_at",
                OrderIncidentsBy::Priority => "priority",
            },
            match opts.order_direction {
                OrderDirection::Asc => "ASC",
                OrderDirection::Desc => "DESC",
            }
        ))
        // $1: organization_id
        .bind(organization_id)
        // $2: statuses
        .bind(&statuses)
        // $3: priorities
        .bind(&priorities)
        // $4: limit
        .bind(opts.limit as i64)
        // $5: offset
        .bind(opts.offset as i64)
        // $6: http_monitor incident_source_type
        .bind(IncidentSourceType::HttpMonitor as i16)
        // $7: http monitor ids
        .bind(&http_monitor_sources_ids)
        // $8: from date
        .bind(opts.from_date)
        // $8: to date
        .bind(opts.to_date)
        .fetch_all(transaction.as_mut())
        .await?;

        let total_filtered_count = rows
            .first()
            .map(|row| row.get::<i64, _>("filtered_count"))
            .unwrap_or_default();

        let incidents = rows
            .into_iter()
            .map(|row| Incident {
                organization_id,
                id: row.get("id"),
                created_at: row.get("created_at"),
                created_by: row.get("created_by"),
                resolved_at: row.get("resolved_at"),
                metadata: row.get::<Option<serde_json::Value>, _>("metadata").into(),
                cause: row
                    .get::<Option<serde_json::Value>, _>("cause")
                    .and_then(|value| serde_json::from_value(value).ok()),
                status: row.get::<i16, _>("status").into(),
                priority: row.get::<i16, _>("priority").into(),
                incident_source_id: row.get::<Uuid, _>("incident_source_id"),
                incident_source_type: row.get::<i16, _>("incident_source_type").into(),
                acknowledged_by: row.get::<Vec<Uuid>, _>("acknowledged_by"),
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
            metadata: record.metadata.into(),
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

    /// Updates the incident with the given ID.
    async fn update_incident(
        &self,
        transaction: &mut Self::Transaction,
        incident: Incident,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE incidents SET
                status = $1,
                priority = $2,
                metadata = $3,
                cause = $4,
                incident_source_type = $5,
                incident_source_id = $6,
                resolved_at = $7
            WHERE organization_id = $8 AND id = $9",
            incident.status as i16,
            incident.priority as i16,
            serde_json::to_value(incident.metadata)?,
            serde_json::to_value(incident.cause)?,
            incident.incident_source_type as i16,
            incident.incident_source_id,
            incident.resolved_at,
            incident.organization_id,
            incident.id
        )
        .execute(transaction.as_mut())
        .await?;

        Ok(())
    }

    /// Deletes an incident with the given ID.
    async fn delete_incident(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM incidents WHERE organization_id = $1 AND id = $2", organization_id, incident_id)
            .execute(transaction.as_mut())
            .await?;
        Ok(())
    }
}
