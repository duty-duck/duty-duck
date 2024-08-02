use chrono::{DateTime, Utc};
use serde::Deserialize;
use ts_rs::TS;
use uuid::Uuid;
use async_trait::async_trait;

use crate::domain::entities::http_monitor::{HttpMonitor, HttpMonitorErrorKind, HttpMonitorStatus};

use super::transactional_repository::TransactionalRepository;

pub struct ListHttpMonitorsOutput {
    pub monitors: Vec<HttpMonitor>,
    pub total_monitors: u32,
    pub total_filtered_monitors: u32,
}

#[async_trait]
pub trait HttpMonitorRepository: TransactionalRepository + Clone + Send + Sync + 'static {
    /// List all the http monitors, return a vector of monitors of size `limit`, along with the total number of monitors
    async fn list_http_monitors(
        &self,
        organization_id: Uuid,
        include_statuses: Vec<HttpMonitorStatus>,
        query: String,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<ListHttpMonitorsOutput>;

    /// Create a new HTTP monitor
    async fn create_http_monitor(&self, monitor: NewHttpMonitor) -> anyhow::Result<Uuid>;

    /// List all the monitors that are due for a refresh
    /// This must be executed inside a transaction. Concurrent transactions will not return the same monitors (monitors that are locked by a transaction will be skipped)
    async fn list_due_http_monitors(
        &self,
        transaction: &mut Self::Transaction,
        limit: u32,
    ) -> anyhow::Result<Vec<HttpMonitor>>;

    async fn update_http_monitor_status(
        &self,
        transaction: &mut Self::Transaction,
        command: UpdateHttpMonitorStatus,
    ) -> anyhow::Result<()>;
}

#[derive(Debug, Deserialize, TS)]
pub struct NewHttpMonitor {
    pub organization_id: Uuid,
    pub url: String,
    pub status: HttpMonitorStatus,
    pub next_ping_at: Option<DateTime<Utc>>,
    pub interval_seconds: u32,
    pub tags: Vec<String>,
}

pub struct UpdateHttpMonitorStatus {
    pub organization_id: Uuid,
    pub monitor_id: Uuid,
    pub status: HttpMonitorStatus,
    pub next_ping_at: Option<DateTime<Utc>>,
    pub status_counter: i16,
    pub error_kind: HttpMonitorErrorKind,
    pub last_http_code: Option<i16>
}
