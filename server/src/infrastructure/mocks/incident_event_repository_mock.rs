use axum::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::{
    entities::incident_event::IncidentEvent,
    ports::{
        incident_event_repository::IncidentEventRepository,
        transactional_repository::{TransactionMock, TransactionalRepository},
    },
};

#[derive(Clone)]
pub struct IncidentEventRepositoryMock {
    pub state: Arc<Mutex<Vec<IncidentEvent>>>,
}

impl IncidentEventRepositoryMock {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl TransactionalRepository for IncidentEventRepositoryMock {
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
impl IncidentEventRepository for IncidentEventRepositoryMock {
    async fn create_incident_event(
        &self,
        _tx: &mut Self::Transaction,
        event: IncidentEvent,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        state.push(event);
        Ok(())
    }

    async fn get_incident_timeline(
        &self,
        organization_id: Uuid,
        incident_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<Vec<IncidentEvent>> {
        let state = self.state.lock().await;

        let filtered_events: Vec<IncidentEvent> = state
            .iter()
            .filter(|e| e.organization_id == organization_id && e.incident_id == incident_id)
            .cloned()
            .collect();

        let start = offset as usize;
        let end = (offset + limit) as usize;

        Ok(
            filtered_events[start.min(filtered_events.len())..end.min(filtered_events.len())]
                .to_vec(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::incident_event::{
        IncidentEventPayload, IncidentEventType, NotificationEventPayload,
    };

    fn create_test_event(
        org_id: Uuid,
        incident_id: Uuid,
        event_type: IncidentEventType,
    ) -> IncidentEvent {
        IncidentEvent {
            organization_id: org_id,
            incident_id,
            user_id: Some(Uuid::new_v4()),
            created_at: Utc::now(),
            event_type,
            event_payload: None,
        }
    }

    #[tokio::test]
    async fn test_create_event_updates_state() -> anyhow::Result<()> {
        let repo = IncidentEventRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let incident_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        let event = create_test_event(org_id, incident_id, IncidentEventType::Creation);
        repo.create_incident_event(&mut tx, event.clone()).await?;

        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);
        assert_eq!(state[0].organization_id, org_id);
        assert_eq!(state[0].incident_id, incident_id);
        assert_eq!(state[0].event_type, IncidentEventType::Creation);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_incident_timeline_with_multiple_events() -> anyhow::Result<()> {
        let repo = IncidentEventRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let incident_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        // Create events for the same incident
        let events = vec![
            create_test_event(org_id, incident_id, IncidentEventType::Creation),
            create_test_event(org_id, incident_id, IncidentEventType::Comment),
            create_test_event(org_id, incident_id, IncidentEventType::Notification),
        ];

        for event in events {
            repo.create_incident_event(&mut tx, event).await?;
        }

        // Create an event for a different incident
        let other_incident_id = Uuid::new_v4();
        let other_event = create_test_event(org_id, other_incident_id, IncidentEventType::Creation);
        repo.create_incident_event(&mut tx, other_event).await?;

        let timeline = repo
            .get_incident_timeline(org_id, incident_id, 10, 0)
            .await?;
        assert_eq!(timeline.len(), 3);
        assert!(timeline.iter().all(|e| e.incident_id == incident_id));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_incident_timeline_pagination() -> anyhow::Result<()> {
        let repo = IncidentEventRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let incident_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        // Create 5 events
        for _ in 0..5 {
            let event = create_test_event(org_id, incident_id, IncidentEventType::Comment);
            repo.create_incident_event(&mut tx, event).await?;
        }

        // Test pagination
        let page1 = repo
            .get_incident_timeline(org_id, incident_id, 2, 0)
            .await?;
        let page2 = repo
            .get_incident_timeline(org_id, incident_id, 2, 2)
            .await?;
        let page3 = repo
            .get_incident_timeline(org_id, incident_id, 2, 4)
            .await?;

        assert_eq!(page1.len(), 2);
        assert_eq!(page2.len(), 2);
        assert_eq!(page3.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_event_with_payload() -> anyhow::Result<()> {
        let repo = IncidentEventRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let incident_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        let mut event = create_test_event(org_id, incident_id, IncidentEventType::Notification);
        event.event_payload = Some(IncidentEventPayload::Notification(
            NotificationEventPayload {
                escalation_level: 1,
                sent_via_email: true,
                sent_via_push_notification: true,
                sent_via_sms: true,
            },
        ));

        repo.create_incident_event(&mut tx, event).await?;

        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);
        assert!(matches!(
            state[0].event_payload,
            Some(IncidentEventPayload::Notification(_))
        ));

        Ok(())
    }

    #[tokio::test]
    async fn test_organization_isolation() -> anyhow::Result<()> {
        let repo = IncidentEventRepositoryMock::new();
        let org_id1 = Uuid::new_v4();
        let org_id2 = Uuid::new_v4();
        let incident_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        // Create events for different organizations
        let event1 = create_test_event(org_id1, incident_id, IncidentEventType::Creation);
        let event2 = create_test_event(org_id2, incident_id, IncidentEventType::Creation);

        repo.create_incident_event(&mut tx, event1).await?;
        repo.create_incident_event(&mut tx, event2).await?;

        let timeline1 = repo
            .get_incident_timeline(org_id1, incident_id, 10, 0)
            .await?;
        let timeline2 = repo
            .get_incident_timeline(org_id2, incident_id, 10, 0)
            .await?;

        assert_eq!(timeline1.len(), 1);
        assert_eq!(timeline2.len(), 1);
        assert_eq!(timeline1[0].organization_id, org_id1);
        assert_eq!(timeline2[0].organization_id, org_id2);

        Ok(())
    }
}
