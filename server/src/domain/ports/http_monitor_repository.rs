use uuid::Uuid;

use crate::domain::entities::http_monitor::HttpMonitor;

pub trait HttpMonitorRepository {
    /// List all the http monitors, return a vector of monitors of size `limit`, along with the total number of monitors
    async fn list_http_monitors(&self, organization_id: Uuid, limit: u32, offset: u32) -> anyhow::Result<(Vec<HttpMonitor>, u64)>;
}