use async_trait::async_trait;
use itertools::Itertools;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    entities::{
        entity_metadata::{
            FilterableMetadata, FilterableMetadataItem, FilterableMetadataValue, MetadataFilter,
        },
        http_monitor::{HttpMonitor, HttpMonitorErrorKind, HttpMonitorStatus},
    },
    ports::{
        http_monitor_repository::{
            self, HttpMonitorRepository, ListHttpMonitorsOutput, NewHttpMonitor,
            UpdateHttpMonitorStatusCommand,
        },
        transactional_repository::TransactionalRepository,
    },
};
use anyhow::*;

#[derive(Clone)]
pub struct HttpMonitorRepositoryAdapter {
    pub pool: PgPool,
}

crate::postgres_transactional_repo!(HttpMonitorRepositoryAdapter);

#[async_trait]
impl HttpMonitorRepository for HttpMonitorRepositoryAdapter {
    #[tracing::instrument(skip(self))]
    async fn get_http_monitor(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        monitor_id: Uuid,
    ) -> anyhow::Result<Option<HttpMonitor>> {
        sqlx::query_as!(
            HttpMonitor,
            "SELECT * FROM http_monitors WHERE  organization_id = $1 AND id = $2",
            organization_id,
            monitor_id,
        )
        .fetch_optional(transaction.as_mut())
        .await
        .with_context(|| "Failed to get single http monitor from the database")
    }

    #[tracing::instrument(skip(self))]
    async fn list_http_monitors(
        &self,
        organization_id: uuid::Uuid,
        include_statuses: Vec<HttpMonitorStatus>,
        query: String,
        metadata_filter: MetadataFilter,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<ListHttpMonitorsOutput> {
        let mut tx = self.begin_transaction().await?;
        let query = format!("%{query}%");
        let statuses = include_statuses
            .into_iter()
            .map(|s| s as i32)
            .collect::<Vec<_>>();
        let metadata_filter = serde_json::to_value(metadata_filter.items)?;

        let rows = sqlx::query!(
            r#"
            WITH filter_conditions AS (
                SELECT 
                    key,
                    jsonb_array_elements_text(value) as filter_value
                FROM jsonb_each($6::jsonb)
            )   
            SELECT *, COUNT(http_monitors.id) OVER () as "filtered_count!" FROM http_monitors  
            WHERE 
            -- filter by organization
            organization_id = $1 
            -- filter by status
            AND status IN (SELECT unnest($2::integer[])) 
            -- filter by url
            AND ($3 = '' or url ilike $3) 
            -- filter by metadata
            AND (
                $6::jsonb = '{}'::jsonb OR
                NOT EXISTS (
                    SELECT 1 FROM filter_conditions fc
                    WHERE NOT EXISTS (
                        SELECT 1 FROM jsonb_each(http_monitors.metadata->'records') m
                        WHERE m.key = fc.key
                        AND (m.value #>> '{}') = fc.filter_value
                    )
                )
            )
            ORDER BY url LIMIT $4 OFFSET $5
            "#,
            organization_id, // $1
            &statuses, // $2
            &query, // $3
            limit as i64, // $4
            offset as i64, // $5
            &metadata_filter, // $6
        )
        .fetch_all(&mut *tx)
        .await?;

        let total_count = sqlx::query!(
            "SELECT count(*) FROM http_monitors WHERE organization_id = $1",
            organization_id
        )
        .fetch_one(&mut *tx)
        .await?
        .count
        .unwrap_or_default();

        let total_filtered_count = rows
            .first()
            .map(|row| row.filtered_count)
            .unwrap_or_default();

        let http_monitors = rows
            .into_iter()
            .map(|row| HttpMonitor {
                organization_id: row.organization_id,
                id: row.id,
                created_at: row.created_at,
                url: row.url,
                first_ping_at: row.first_ping_at,
                next_ping_at: row.next_ping_at,
                last_ping_at: row.last_ping_at,
                last_status_change_at: row.last_status_change_at,
                recovery_confirmation_threshold: row.recovery_confirmation_threshold,
                downtime_confirmation_threshold: row.downtime_confirmation_threshold,
                interval_seconds: row.interval_seconds as i64,
                last_http_code: row.last_http_code,
                status: row.status.into(),
                status_counter: row.status_counter,
                error_kind: row.error_kind.into(),
                metadata: row.metadata.into(),
                email_notification_enabled: row.email_notification_enabled,
                push_notification_enabled: row.push_notification_enabled,
                sms_notification_enabled: row.sms_notification_enabled,
                archived_at: row.archived_at,
                request_headers: row.request_headers.into(),
                request_timeout_ms: row.request_timeout_ms,
            })
            .collect::<Vec<_>>();

        Ok(ListHttpMonitorsOutput {
            monitors: http_monitors,
            total_monitors: total_count as u32,
            total_filtered_monitors: total_filtered_count as u32,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn create_http_monitor(
        &self,
        monitor: http_monitor_repository::NewHttpMonitor,
    ) -> anyhow::Result<Uuid> {
        let metadata = serde_json::to_value(monitor.metadata)?;
        let new_monitor_id = sqlx::query!(
            "insert into http_monitors (
                organization_id, 
                url, 
                status, 
                status_counter, 
                next_ping_at, 
                interval_seconds, 
                error_kind, 
                metadata,
                downtime_confirmation_threshold,
                recovery_confirmation_threshold,
                email_notification_enabled,
                push_notification_enabled,
                sms_notification_enabled
            ) 
            values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            returning id",
            monitor.organization_id,
            monitor.url,
            monitor.status as i16,
            0,
            monitor.next_ping_at,
            monitor.interval_seconds as i64,
            HttpMonitorErrorKind::None as i16,
            &metadata,
            monitor.downtime_confirmation_threshold as i64,
            monitor.recovery_confirmation_threshold as i64,
            monitor.email_notification_enabled,
            monitor.push_notification_enabled,
            monitor.sms_notification_enabled,
        )
        .fetch_one(&self.pool)
        .await?
        .id;
        Ok(new_monitor_id)
    }

    async fn list_due_http_monitors(
        &self,
        transaction: &mut Self::Transaction,
        limit: u32,
    ) -> anyhow::Result<Vec<HttpMonitor>> {
        let http_monitors = sqlx::query_as!(
            HttpMonitor,
            "SELECT * FROM  http_monitors
            WHERE status != $1
            AND next_ping_at <= NOW()
            FOR UPDATE SKIP LOCKED
            LIMIT $2",
            HttpMonitorStatus::Inactive as i32,
            limit as i64,
        )
        .fetch_all(transaction.as_mut())
        .await?;

        Ok(http_monitors)
    }

    async fn update_http_monitor_status(
        &self,
        transaction: &mut Self::Transaction,
        command: UpdateHttpMonitorStatusCommand,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE http_monitors SET 
                status = $1,
                next_ping_at = $2, 
                status_counter = $3,
                error_kind = $4,
                last_http_code = $5,
                last_ping_at = now(),
                first_ping_at = coalesce(first_ping_at, now()),
                last_status_change_at = $6
            WHERE organization_id = $7 and id = $8",
            command.status as i16,
            command.next_ping_at,
            command.status_counter,
            command.error_kind as i16,
            command.last_http_code,
            command.last_status_change_at,
            command.organization_id,
            command.monitor_id,
        )
        .execute(transaction.as_mut())
        .await?;

        Ok(())
    }

    async fn update_http_monitor(&self, id: Uuid, monitor: NewHttpMonitor) -> anyhow::Result<bool> {
        let metadata = serde_json::to_value(monitor.metadata)?;
        let request_headers = serde_json::to_value(monitor.request_headers)?;

        let result = sqlx::query!(
            "UPDATE http_monitors SET 
                url = $1,
                status = $2,
                next_ping_at = $3, 
                metadata = $4,
                interval_seconds = $5,
                recovery_confirmation_threshold = $6,
                downtime_confirmation_threshold = $7,
                email_notification_enabled = $8,
                push_notification_enabled = $9,
                sms_notification_enabled = $10,
                request_headers = $11,
                request_timeout_ms = $12,
                organization_id = $13
            WHERE organization_id = $13 and id = $14",
            monitor.url, // $1
            monitor.status as i16, // $2
            monitor.next_ping_at, // $3
            &metadata, // $4
            monitor.interval_seconds as i64, // $5
            monitor.recovery_confirmation_threshold as i64, // $6
            monitor.downtime_confirmation_threshold as i64, // $7
            monitor.email_notification_enabled, // $8
            monitor.push_notification_enabled, // $9
            monitor.sms_notification_enabled, // $10
            &request_headers, // $11
            monitor.request_timeout_ms, // $12
            monitor.organization_id, // $13
            id, // $14
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get the filterable metadata for all the monitors of an organization
    async fn get_filterable_metadata(
        &self,
        organization_id: Uuid,
    ) -> anyhow::Result<FilterableMetadata> {
        let records = sqlx::query!(
            r#"
            WITH RECURSIVE 
            json_keys AS (
                SELECT DISTINCT
                    key,
                    value #>> '{}' as value_str
                FROM http_monitors,
                jsonb_each(metadata -> 'records') as fields(key, value)
                WHERE http_monitors.organization_id = $1 AND http_monitors.status != $2
            )
            SELECT 
            key as "key!",
            value_str as "value!",
            COUNT(*) OVER (PARTITION BY key, value_str) as "value_occurrence_count!"
            FROM json_keys
            ORDER BY key, value_str;
            "#,
            organization_id,
            HttpMonitorStatus::Archived as i16,
        )
        .fetch_all(&self.pool)
        .await?;

        let items = records
            .into_iter()
            .chunk_by(|r| r.key.clone())
            .into_iter()
            .map(|(key, chunk)| {
                let distinct_values: Vec<FilterableMetadataValue> = chunk
                    .map(|r| FilterableMetadataValue {
                        value: r.value,
                        value_count: r.value_occurrence_count as u64,
                    })
                    .collect();
                FilterableMetadataItem {
                    key,
                    key_cardinality: distinct_values.len() as u64,
                    distinct_values,
                }
            })
            .collect();

        Ok(FilterableMetadata { items })
    }
}
