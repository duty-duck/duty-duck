use axum::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::{
    entities::incident_notification::IncidentNotification,
    ports::{
        incident_notification_repository::IncidentNotificationRepository,
        transactional_repository::{TransactionMock, TransactionalRepository},
    },
};

#[derive(Clone)]
pub struct IncidentNotificationRepositoryMock {
    pub state: Arc<Mutex<Vec<IncidentNotification>>>,
}

impl IncidentNotificationRepositoryMock {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl TransactionalRepository for IncidentNotificationRepositoryMock {
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
impl IncidentNotificationRepository for IncidentNotificationRepositoryMock {
    async fn get_next_notifications_to_send(
        &self,
        _tx: &mut Self::Transaction,
        limit: u32,
    ) -> anyhow::Result<Vec<IncidentNotification>> {
        let mut state = self.state.lock().await;
        let now = Utc::now();

        // Sort by due date to ensure we get the most urgent notifications first
        state.sort_by(|a, b| a.notification_due_at.cmp(&b.notification_due_at));

        let due_notifications: Vec<IncidentNotification> = state
            .iter()
            .filter(|n| n.notification_due_at <= now)
            .take(limit as usize)
            .cloned()
            .collect();

        // Remove the notifications that we're returning (simulating the FOR UPDATE SKIP LOCKED behavior)
        state.retain(|n| {
            !due_notifications.iter().any(|dn| {
                dn.organization_id == n.organization_id
                    && dn.incident_id == n.incident_id
                    && dn.escalation_level == n.escalation_level
            })
        });

        Ok(due_notifications)
    }

    async fn upsert_incident_notification(
        &self,
        _tx: &mut Self::Transaction,
        notification: IncidentNotification,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;

        let existing_idx = state.iter().position(|n| {
            n.organization_id == notification.organization_id
                && n.incident_id == notification.incident_id
                && n.escalation_level == notification.escalation_level
        });

        match existing_idx {
            Some(idx) => state[idx] = notification,
            None => state.push(notification),
        }

        Ok(())
    }

    async fn cancel_all_notifications_for_incident(
        &self,
        _tx: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        state.retain(|n| !(n.organization_id == organization_id && n.incident_id == incident_id));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::time::Duration;

    use super::*;
    use crate::domain::entities::http_monitor::HttpMonitorErrorKind;
    use crate::domain::entities::incident::{HttpMonitorIncidentCause, HttpMonitorIncidentCausePing, IncidentCause};
    use crate::domain::entities::incident_notification::{
        IncidentNotificationPayload, IncidentNotificationType,
    };
    use chrono::{DateTime, Utc};

    fn create_test_notification(
        org_id: Uuid,
        incident_id: Uuid,
        escalation_level: i16,
        due_at: DateTime<Utc>,
    ) -> IncidentNotification {
        IncidentNotification {
            organization_id: org_id,
            incident_id,
            escalation_level,
            notification_type: IncidentNotificationType::IncidentCreation,
            notification_due_at: due_at,
            notification_payload: IncidentNotificationPayload {
                incident_cause: IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
                    last_ping: HttpMonitorIncidentCausePing {
                        error_kind: HttpMonitorErrorKind::Timeout,
                        http_code: None,
                    },
                    previous_pings: HashSet::new(),
                }),
                incident_http_monitor_url: Some("https://example.com".to_string()),
            },
            send_sms: true,
            send_push_notification: true,
            send_email: true,
        }
    }

    #[tokio::test]
    async fn test_upsert_notification() -> anyhow::Result<()> {
        let repo = IncidentNotificationRepositoryMock::new();
        let mut tx = repo.begin_transaction().await?;
        let org_id = Uuid::new_v4();
        let incident_id = Uuid::new_v4();

        let notification = create_test_notification(org_id, incident_id, 1, Utc::now());
        repo.upsert_incident_notification(&mut tx, notification.clone())
            .await?;

        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);
        assert_eq!(state[0].organization_id, org_id);
        assert_eq!(state[0].incident_id, incident_id);
        assert_eq!(state[0].escalation_level, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_updates_existing_notification() -> anyhow::Result<()> {
        let repo = IncidentNotificationRepositoryMock::new();
        let mut tx = repo.begin_transaction().await?;
        let org_id = Uuid::new_v4();
        let incident_id = Uuid::new_v4();

        let mut notification = create_test_notification(org_id, incident_id, 1, Utc::now());
        repo.upsert_incident_notification(&mut tx, notification.clone())
            .await?;

        // Update the notification
        notification.send_sms = false;
        repo.upsert_incident_notification(&mut tx, notification)
            .await?;

        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);
        assert!(!state[0].send_sms);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_next_notifications_to_send() -> anyhow::Result<()> {
        let repo = IncidentNotificationRepositoryMock::new();
        let mut tx = repo.begin_transaction().await?;
        let org_id = Uuid::new_v4();
        let incident_id = Uuid::new_v4();

        let now = Utc::now();
        let notifications = vec![
            create_test_notification(org_id, incident_id, 1, now - Duration::from_secs(5 * 60)),
            create_test_notification(org_id, incident_id, 2, now - Duration::from_secs(60)),
            create_test_notification(org_id, incident_id, 3, now + Duration::from_secs(5 * 60)),
        ];

        for notification in notifications {
            repo.upsert_incident_notification(&mut tx, notification)
                .await?;
        }

        let due_notifications = repo.get_next_notifications_to_send(&mut tx, 2).await?;
        assert_eq!(due_notifications.len(), 2);
        assert!(due_notifications
            .iter()
            .all(|n| n.notification_due_at <= now));

        // Verify the notifications were removed from state
        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);
        assert!(state[0].notification_due_at > now);

        Ok(())
    }

    #[tokio::test]
    async fn test_cancel_all_notifications_for_incident() -> anyhow::Result<()> {
        let repo = IncidentNotificationRepositoryMock::new();
        let mut tx = repo.begin_transaction().await?;
        let org_id = Uuid::new_v4();
        let incident_id1 = Uuid::new_v4();
        let incident_id2 = Uuid::new_v4();

        // Create notifications for two different incidents
        let notifications = vec![
            create_test_notification(org_id, incident_id1, 1, Utc::now()),
            create_test_notification(org_id, incident_id1, 2, Utc::now()),
            create_test_notification(org_id, incident_id2, 1, Utc::now()),
        ];

        for notification in notifications {
            repo.upsert_incident_notification(&mut tx, notification)
                .await?;
        }

        // Cancel notifications for incident1
        repo.cancel_all_notifications_for_incident(&mut tx, org_id, incident_id1)
            .await?;

        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);
        assert_eq!(state[0].incident_id, incident_id2);

        Ok(())
    }

    #[tokio::test]
    async fn test_organization_isolation() -> anyhow::Result<()> {
        let repo = IncidentNotificationRepositoryMock::new();
        let mut tx = repo.begin_transaction().await?;
        let org_id1 = Uuid::new_v4();
        let org_id2 = Uuid::new_v4();
        let incident_id = Uuid::new_v4();

        let notifications = vec![
            create_test_notification(org_id1, incident_id, 1, Utc::now()),
            create_test_notification(org_id2, incident_id, 1, Utc::now()),
        ];

        for notification in notifications {
            repo.upsert_incident_notification(&mut tx, notification)
                .await?;
        }

        repo.cancel_all_notifications_for_incident(&mut tx, org_id1, incident_id)
            .await?;

        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);
        assert_eq!(state[0].organization_id, org_id2);

        Ok(())
    }
}
