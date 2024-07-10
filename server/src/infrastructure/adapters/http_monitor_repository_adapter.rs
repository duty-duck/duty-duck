use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    entities::http_monitor::HttpMonitor,
    ports::http_monitor_repository::{self, HttpMonitorRepository},
};
use anyhow::*;

#[derive(Clone)]
pub struct HttpMonitorRepositoryAdapter {
    pub pool: PgPool,
}

impl HttpMonitorRepository for HttpMonitorRepositoryAdapter {
    #[tracing::instrument(skip(self))]
    async fn list_http_monitors(
        &self,
        organization_id: uuid::Uuid,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<(Vec<HttpMonitor>, u64)> {
        let mut tx = self.pool.begin().await?;

        let http_monitors = sqlx::query_as!(
            HttpMonitor,
            "SELECT * FROM http_monitors WHERE organization_id = $1 LIMIT $2 OFFSET $3 ",
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
            "insert into http_monitors (organization_id, url, status, status_counter, next_ping_at, interval_seconds, tags) 
            values ($1, $2, $3, $4, $5, $6, $7)
            returning id",
            monitor.organization_id,
            monitor.url,
            monitor.status as i16,
            0,
            monitor.next_ping_at,
            monitor.interval_seconds as i64,
            &monitor.tags
        ).fetch_one(&self.pool).await?.id;
        Ok(new_monitor_id)
    }
}
