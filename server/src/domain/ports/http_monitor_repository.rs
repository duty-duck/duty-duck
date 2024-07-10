use chrono::{DateTime, Utc};
use serde::Deserialize;
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::entities::http_monitor::{HttpMonitor, HttpMonitorStatus};

pub trait HttpMonitorRepository {
    /// List all the http monitors, return a vector of monitors of size `limit`, along with the total number of monitors
    async fn list_http_monitors(
        &self,
        organization_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<(Vec<HttpMonitor>, u64)>;

    async fn create_http_monitor(&self, monitor: NewHttpMonitor) -> anyhow::Result<Uuid>;
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
