use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    entities::http_monitor::{HttpMonitor, HttpMonitorStatus},
    ports::{
        http_monitor_repository::{
            HttpMonitorRepository, ListHttpMonitorsOutput, NewHttpMonitor,
            UpdateHttpMonitorStatusCommand,
        },
        transactional_repository::{TransactionMock, TransactionalRepository},
    },
};

#[derive(Clone)]
pub struct HttpMonitorRepositoryMock {
    pub state: Arc<Mutex<Vec<HttpMonitor>>>,
}

impl HttpMonitorRepositoryMock {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl TransactionalRepository for HttpMonitorRepositoryMock {
    type Transaction = TransactionMock;

    async fn begin_transaction(&self) -> anyhow::Result<Self::Transaction> {
        Ok(TransactionMock)
    }

    async fn commit_transaction(&self, _transaction: Self::Transaction) -> anyhow::Result<()> {
        Ok(())
    }

    async fn rollback_transaction(&self, _transaction: Self::Transaction) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl HttpMonitorRepository for HttpMonitorRepositoryMock {
    async fn get_http_monitor(
        &self,
        _transaction: &mut Self::Transaction,
        organization_id: Uuid,
        monitor_id: Uuid,
    ) -> anyhow::Result<Option<HttpMonitor>> {
        let state = self.state.lock().await;
        Ok(state
            .iter()
            .find(|m| m.id == monitor_id && m.organization_id == organization_id)
            .cloned())
    }

    async fn list_http_monitors(
        &self,
        organization_id: Uuid,
        include_statuses: Vec<HttpMonitorStatus>,
        query: String,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<ListHttpMonitorsOutput> {
        let state = self.state.lock().await;
        
        let total_monitors = state
            .iter()
            .filter(|m| m.organization_id == organization_id)
            .count() as u32;

        let filtered_monitors: Vec<HttpMonitor> = state
            .iter()
            .filter(|m| m.organization_id == organization_id)
            .filter(|m| include_statuses.is_empty() || include_statuses.contains(&m.status))
            .filter(|m| query.is_empty() || m.url.to_lowercase().contains(&query.to_lowercase()))
            .cloned()
            .collect();

        let total_filtered_monitors = filtered_monitors.len() as u32;
        
        let start = offset as usize;
        let end = (offset + limit) as usize;
        let monitors = filtered_monitors[start.min(filtered_monitors.len())..end.min(filtered_monitors.len())].to_vec();

        Ok(ListHttpMonitorsOutput {
            monitors,
            total_monitors,
            total_filtered_monitors,
        })
    }

    async fn create_http_monitor(&self, monitor: NewHttpMonitor) -> anyhow::Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let monitor = HttpMonitor {
            id,
            organization_id: monitor.organization_id,
            created_at: now,
            url: monitor.url,
            first_ping_at: None,
            next_ping_at: monitor.next_ping_at,
            last_ping_at: None,
            last_status_change_at: now,
            recovery_confirmation_threshold: monitor.recovery_confirmation_threshold as i16,
            downtime_confirmation_threshold: monitor.downtime_confirmation_threshold as i16,
            interval_seconds: monitor.interval_seconds as i64,
            last_http_code: None,
            status: monitor.status,
            status_counter: 0,
            error_kind: crate::domain::entities::http_monitor::HttpMonitorErrorKind::None,
            metadata: monitor.metadata,
            email_notification_enabled: monitor.email_notification_enabled,
            push_notification_enabled: monitor.push_notification_enabled,
            sms_notification_enabled: monitor.sms_notification_enabled,
            archived_at: None,
            request_headers: monitor.request_headers,
            request_timeout_ms: monitor.request_timeout_ms,
        };

        let mut state = self.state.lock().await;
        state.push(monitor);
        Ok(id)
    }

    async fn update_http_monitor(&self, id: Uuid, monitor: NewHttpMonitor) -> anyhow::Result<bool> {
        let mut state = self.state.lock().await;
        
        if let Some(existing) = state.iter_mut().find(|m| m.id == id && m.organization_id == monitor.organization_id) {
            existing.url = monitor.url;
            existing.status = monitor.status;
            existing.next_ping_at = monitor.next_ping_at;
            existing.metadata = monitor.metadata;
            existing.interval_seconds = monitor.interval_seconds as i64;
            existing.recovery_confirmation_threshold = monitor.recovery_confirmation_threshold as i16;
            existing.downtime_confirmation_threshold = monitor.downtime_confirmation_threshold as i16;
            existing.email_notification_enabled = monitor.email_notification_enabled;
            existing.push_notification_enabled = monitor.push_notification_enabled;
            existing.sms_notification_enabled = monitor.sms_notification_enabled;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn list_due_http_monitors(
        &self,
        _transaction: &mut Self::Transaction,
        limit: u32,
    ) -> anyhow::Result<Vec<HttpMonitor>> {
        let state = self.state.lock().await;
        let now = Utc::now();
        
        let due_monitors: Vec<HttpMonitor> = state
            .iter()
            .filter(|m| m.status != HttpMonitorStatus::Inactive)
            .filter(|m| m.next_ping_at.map(|t| t <= now).unwrap_or(false))
            .take(limit as usize)
            .cloned()
            .collect();

        Ok(due_monitors)
    }

    async fn update_http_monitor_status(
        &self,
        _transaction: &mut Self::Transaction,
        command: UpdateHttpMonitorStatusCommand,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        
        if let Some(monitor) = state.iter_mut().find(|m| {
            m.id == command.monitor_id && m.organization_id == command.organization_id
        }) {
            monitor.status = command.status;
            monitor.next_ping_at = command.next_ping_at;
            monitor.status_counter = command.status_counter;
            monitor.error_kind = command.error_kind;
            monitor.last_http_code = command.last_http_code;
            monitor.last_status_change_at = command.last_status_change_at;
            monitor.last_ping_at = Some(Utc::now());
            if monitor.first_ping_at.is_none() {
                monitor.first_ping_at = Some(Utc::now());
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{entity_metadata::EntityMetadata, http_monitor::RequestHeaders};

    fn create_test_monitor(org_id: Uuid, url: &str, status: HttpMonitorStatus) -> NewHttpMonitor {
        NewHttpMonitor {
            organization_id: org_id,
            url: url.to_string(),
            status,
            next_ping_at: Some(Utc::now()),
            interval_seconds: 60,
            metadata: EntityMetadata::default(),
            recovery_confirmation_threshold: 3,
            downtime_confirmation_threshold: 3,
            email_notification_enabled: true,
            push_notification_enabled: false,
            sms_notification_enabled: false,
            request_headers: RequestHeaders::default(),
            request_timeout_ms: 2000,
        }
    }

    #[tokio::test]
    async fn test_create_monitor_updates_state() -> anyhow::Result<()> {
        let repo = HttpMonitorRepositoryMock::new();
        let org_id = Uuid::new_v4();
        
        let monitor = create_test_monitor(org_id, "https://example.com", HttpMonitorStatus::Up);
        let id = repo.create_http_monitor(monitor).await?;
        
        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);
        
        let created_monitor = &state[0];
        assert_eq!(created_monitor.id, id);
        assert_eq!(created_monitor.organization_id, org_id);
        assert_eq!(created_monitor.url, "https://example.com");
        assert_eq!(created_monitor.status, HttpMonitorStatus::Up);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_list_monitors_with_status_filter() -> anyhow::Result<()> {
        let repo = HttpMonitorRepositoryMock::new();
        let org_id = Uuid::new_v4();
        
        // Create monitors with different statuses
        let monitors = vec![
            create_test_monitor(org_id, "https://up.com", HttpMonitorStatus::Up),
            create_test_monitor(org_id, "https://down.com", HttpMonitorStatus::Down),
            create_test_monitor(org_id, "https://inactive.com", HttpMonitorStatus::Inactive),
        ];
        
        for monitor in monitors {
            repo.create_http_monitor(monitor).await?;
        }
        
        // Test filtering by Up status
        let result = repo.list_http_monitors(
            org_id,
            vec![HttpMonitorStatus::Up],
            String::new(),
            10,
            0
        ).await?;
        
        assert_eq!(result.monitors.len(), 1);
        assert_eq!(result.monitors[0].url, "https://up.com");
        assert_eq!(result.total_monitors, 3);
        assert_eq!(result.total_filtered_monitors, 1);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_list_monitors_with_url_search() -> anyhow::Result<()> {
        let repo = HttpMonitorRepositoryMock::new();
        let org_id = Uuid::new_v4();
        
        // Create monitors with different URLs
        let monitors = vec![
            create_test_monitor(org_id, "https://api.example.com", HttpMonitorStatus::Up),
            create_test_monitor(org_id, "https://web.example.com", HttpMonitorStatus::Up),
            create_test_monitor(org_id, "https://different.com", HttpMonitorStatus::Up),
        ];
        
        for monitor in monitors {
            repo.create_http_monitor(monitor).await?;
        }
        
        // Test searching for "example"
        let result = repo.list_http_monitors(
            org_id,
            vec![],
            "example".to_string(),
            10,
            0
        ).await?;
        
        assert_eq!(result.monitors.len(), 2);
        assert_eq!(result.total_monitors, 3);
        assert_eq!(result.total_filtered_monitors, 2);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_list_monitors_pagination() -> anyhow::Result<()> {
        let repo = HttpMonitorRepositoryMock::new();
        let org_id = Uuid::new_v4();
        
        // Create multiple monitors
        for i in 1..=5 {
            let monitor = create_test_monitor(
                org_id,
                &format!("https://test{}.com", i),
                HttpMonitorStatus::Up
            );
            repo.create_http_monitor(monitor).await?;
        }
        
        // Test pagination with limit 2
        let page1 = repo.list_http_monitors(
            org_id,
            vec![],
            String::new(),
            2,
            0
        ).await?;
        
        let page2 = repo.list_http_monitors(
            org_id,
            vec![],
            String::new(),
            2,
            2
        ).await?;
        
        assert_eq!(page1.monitors.len(), 2);
        assert_eq!(page2.monitors.len(), 2);
        assert_ne!(page1.monitors[0].id, page2.monitors[0].id);
        assert_eq!(page1.total_monitors, 5);
        assert_eq!(page2.total_monitors, 5);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_list_due_monitors() -> anyhow::Result<()> {
        let repo = HttpMonitorRepositoryMock::new();
        let org_id = Uuid::new_v4();
        
        // Create one monitor due now and one due in the future
        let mut monitor1 = create_test_monitor(org_id, "https://due-now.com", HttpMonitorStatus::Up);
        monitor1.next_ping_at = Some(Utc::now() - chrono::Duration::minutes(1));
        
        let mut monitor2 = create_test_monitor(org_id, "https://due-later.com", HttpMonitorStatus::Up);
        monitor2.next_ping_at = Some(Utc::now() + chrono::Duration::minutes(5));
        
        repo.create_http_monitor(monitor1).await?;
        repo.create_http_monitor(monitor2).await?;
        
        let mut tx = repo.begin_transaction().await?;
        let due_monitors = repo.list_due_http_monitors(&mut tx, 10).await?;
        
        assert_eq!(due_monitors.len(), 1);
        assert_eq!(due_monitors[0].url, "https://due-now.com");
        
        Ok(())
    }
}