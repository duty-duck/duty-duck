use uuid::Uuid;

use crate::domain::entities::http_monitor::HttpMonitor;

pub trait HttpMonitorRepository {
    async fn list_http_monitors(&self, organization_id: Uuid, limit: u32, offset: u32) -> anyhow::Result<Vec<HttpMonitor>>;
}