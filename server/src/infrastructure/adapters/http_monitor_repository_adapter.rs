use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{
    entities::http_monitor::{HttpMonitor, HttpMonitorErrorKind, HttpMonitorStatus},
    ports::{
        http_monitor_repository::{self, HttpMonitorRepository, UpdateHttpMonitorStatus},
        transactional_repository::TransactionalRepository,
    },
};
use anyhow::*;

#[derive(Clone)]
pub struct HttpMonitorRepositoryAdapter {
    pub pool: PgPool,
}

#[async_trait]
impl TransactionalRepository for HttpMonitorRepositoryAdapter {
    type Transaction = Transaction<'static, Postgres>;

    async fn begin_transaction(&self) -> anyhow::Result<Self::Transaction> {
        self.pool
            .begin()
            .await
            .with_context(|| "Cannot begin transaction")
    }

    async fn rollback_transaction(&self, tx: Self::Transaction) -> anyhow::Result<()> {
        tx.rollback()
            .await
            .with_context(|| "Cannot rollback transaction")
    }

    async fn commit_transaction(&self, tx: Self::Transaction) -> anyhow::Result<()> {
        tx.commit()
            .await
            .with_context(|| "Cannot commit transaction")
    }
}

#[async_trait]
impl HttpMonitorRepository for HttpMonitorRepositoryAdapter {
    #[tracing::instrument(skip(self))]
    async fn list_http_monitors(
        &self,
        organization_id: uuid::Uuid,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<(Vec<HttpMonitor>, u64)> {
        let mut tx = self.begin_transaction().await?;

        let http_monitors = sqlx::query_as!(
            HttpMonitor,
            "SELECT * FROM http_monitors WHERE organization_id = $1 LIMIT $2 OFFSET $3",
            organization_id,
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

        Ok((http_monitors, total_count as u64))
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
            "UPDATE http_monitors SET status = $1, next_ping_at = $2, status_counter = $3, error_kind = $4 WHERE organization_id = $5 and id = $6",
            command.status as i16,
            command.next_ping_at,
            command.status_counter,
            command.error_kind as i16,
            command.organization_id,
            command.monitor_id,
        ).execute(transaction.as_mut()).await?;

        Ok(())
    }
}
