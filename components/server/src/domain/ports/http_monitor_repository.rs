use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::{entity_metadata::{EntityMetadata, FilterableMetadata, MetadataFilter}, http_monitor::{HttpMonitor, HttpMonitorErrorKind, HttpMonitorStatus, RequestHeaders}};

use super::transactional_repository::TransactionalRepository;

#[async_trait]
pub trait HttpMonitorRepository: TransactionalRepository + Clone + Send + Sync + 'static {
    async fn get_http_monitor(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        monitor_id: Uuid,
    ) -> anyhow::Result<Option<HttpMonitor>>;

    /// List all the http monitors, return a vector of monitors of size `limit`, along with the total number of monitors
    async fn list_http_monitors(
        &self,
        organization_id: Uuid,
        include_statuses: Vec<HttpMonitorStatus>,
        query: String,
        metadata_filter: MetadataFilter,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<ListHttpMonitorsOutput>;

    /// Create a new HTTP monitor
    async fn create_http_monitor(&self, monitor: NewHttpMonitor) -> anyhow::Result<Uuid>;

    /// Update an HTTP monitor, returns true if the monitor existed, or false if the monitor did not exist
    /// Used by the public API to allow users to update HTTP monitors
    async fn update_http_monitor(
        &self,
        transaction: &mut Self::Transaction,
        id: Uuid,
        monitor: NewHttpMonitor,
    ) -> anyhow::Result<bool>;

    /// List all the monitors that are due for a refresh
    /// This must be executed inside a transaction. Concurrent transactions will not return the same monitors (monitors that are locked by a transaction will be skipped)
    async fn list_due_http_monitors(
        &self,
        transaction: &mut Self::Transaction,
        limit: u32,
    ) -> anyhow::Result<Vec<HttpMonitor>>;

    /// Used by internal use cases to update some fields of an HTTP monitor
    /// Not used by the public API
    async fn update_http_monitor_status(
        &self,
        transaction: &mut Self::Transaction,
        command: UpdateHttpMonitorStatusCommand,
    ) -> anyhow::Result<()>;

    /// Get the filterable metadata for all the monitors of an organization
    async fn get_filterable_metadata(&self, organization_id: Uuid) -> anyhow::Result<FilterableMetadata>;
}

#[derive(Debug)]
pub struct NewHttpMonitor {
    pub organization_id: Uuid,
    pub url: String,
    pub status: HttpMonitorStatus,
    pub next_ping_at: Option<DateTime<Utc>>,
    pub interval_seconds: u32,
    pub metadata: EntityMetadata,
    pub recovery_confirmation_threshold: u32,
    pub downtime_confirmation_threshold: u32,
    pub email_notification_enabled: bool,
    pub push_notification_enabled: bool,
    pub sms_notification_enabled: bool,
    pub request_headers: RequestHeaders,
    pub request_timeout_ms: i32,
}

#[derive(Debug)]
pub struct UpdateHttpMonitorStatusCommand {
    pub organization_id: Uuid,
    pub monitor_id: Uuid,
    pub status: HttpMonitorStatus,
    pub next_ping_at: Option<DateTime<Utc>>,
    pub last_status_change_at: DateTime<Utc>,
    pub status_counter: i16,
    pub error_kind: HttpMonitorErrorKind,
    /// Used to update the last HTTP code received when the monitor is pinged
    /// If None, the last http code will be removed
    pub last_http_code: Option<i16>,
    pub archived_at: Option<DateTime<Utc>>,
}

pub struct ListHttpMonitorsOutput {
    pub monitors: Vec<HttpMonitor>,
    pub total_monitors: u32,
    pub total_filtered_monitors: u32,
}
