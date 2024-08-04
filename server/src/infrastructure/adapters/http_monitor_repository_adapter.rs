use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    entities::http_monitor::{HttpMonitor, HttpMonitorErrorKind, HttpMonitorStatus},
    ports::{
        http_monitor_repository::{
            self, HttpMonitorRepository, ListHttpMonitorsOutput, UpdateHttpMonitorStatus,
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
    async fn list_http_monitors(
        &self,
        organization_id: uuid::Uuid,
        include_statuses: Vec<HttpMonitorStatus>,
        query: String,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<ListHttpMonitorsOutput> {
        let mut tx = self.begin_transaction().await?;
        let query = format!("%{query}%");
        let statuses = include_statuses
            .into_iter()
            .map(|s| s as i32)
            .collect::<Vec<_>>();

        let http_monitors = 
            sqlx::query_as!(
                HttpMonitor,
                "SELECT * FROM http_monitors 
                WHERE organization_id = $1 AND status IN (SELECT unnest($2::integer[])) AND ($3 = '' or url ilike $3) 
                ORDER BY url LIMIT $4 OFFSET $5",
                organization_id,
                &statuses,
                &query,
                limit as i64,
                offset as i64
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

        let total_filtered_count = sqlx::query!(
            "SELECT count(*) FROM http_monitors WHERE organization_id = $1 AND status IN (SELECT unnest($2::integer[])) AND ($3 = '' or url ilike $3)",
            organization_id,
            &statuses,
            &query
        )
        .fetch_one(&mut *tx)
        .await?
        .count
        .unwrap_or_default();

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
        let new_monitor_id = sqlx::query!(
            "insert into http_monitors (organization_id, url, status, status_counter, next_ping_at, interval_seconds, error_kind, tags) 
            values ($1, $2, $3, $4, $5, $6, $7, $8)
            returning id",
            monitor.organization_id,
            monitor.url,
            monitor.status as i16,
            0,
            monitor.next_ping_at,
            monitor.interval_seconds as i64,
            HttpMonitorErrorKind::None as i16,
            &monitor.tags
        ).fetch_one(&self.pool).await?.id;
        Ok(new_monitor_id)
    }

    async fn list_due_http_monitors(
        &self,
        transaction: &mut Self::Transaction,
        limit: u32,
    ) -> anyhow::Result<Vec<HttpMonitor>> {
        let http_monitors = sqlx::query_as!(
            HttpMonitor,
            "SELECT * FROM http_monitors
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
        command: UpdateHttpMonitorStatus,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "UPDATE http_monitors SET 
                status = $1,
                next_ping_at = $2, 
                status_counter = $3,
                error_kind = $4,
                last_http_code = $5,
                last_ping_at = now(),
                first_ping_at = coalesce(first_ping_at, now())
            WHERE organization_id = $6 and id = $7",
            command.status as i16,
            command.next_ping_at,
            command.status_counter,
            command.error_kind as i16,
            command.last_http_code,
            command.organization_id,
            command.monitor_id,
        )
        .execute(transaction.as_mut())
        .await?;

        Ok(())
    }
}
