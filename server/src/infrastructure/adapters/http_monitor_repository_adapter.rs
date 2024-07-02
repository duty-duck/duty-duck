use sqlx::PgPool;

use crate::domain::{
    entities::http_monitor::HttpMonitor, ports::http_monitor_repository::HttpMonitorRepository,
};
use anyhow::*;

#[derive(Clone)]
pub struct HttpMonitorRepositoryAdapter {
    pub pool: PgPool,
}

impl HttpMonitorRepository for HttpMonitorRepositoryAdapter {
    async fn list_http_monitors(
        &self,
        organization_id: uuid::Uuid,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<Vec<HttpMonitor>> {
        sqlx::query_as!(
            HttpMonitor,
            "SELECT * FROM http_monitors WHERE organization_id = $1 LIMIT $2 OFFSET $3",
            organization_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .with_context(|| "Cannot list http monitors")
    }
}
