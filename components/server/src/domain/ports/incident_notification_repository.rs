use uuid::Uuid;

use crate::domain::entities::incident_notification::IncidentNotification;

use super::transactional_repository::TransactionalRepository;

#[async_trait::async_trait]
pub trait IncidentNotificationRepository:
    TransactionalRepository + Clone + Send + Sync + 'static
{
    /// Returns a list notifications that are due to be sent to users
    /// This must be executed inside a transaction.
    /// Concurrent transactions will not return the same incidents (incidents that are locked by a transaction will be skipped)
    /// When the transaction is committed, the rows in the database are deleted, so they won't be selected again
    async fn get_next_notifications_to_send(
        &self,
        tx: &mut Self::Transaction,
        limit: u32,
    ) -> anyhow::Result<Vec<IncidentNotification>>;

    /// Inserts a new notification or updates an existing one
    async fn upsert_incident_notification(
        &self,
        tx: &mut Self::Transaction,
        notification: IncidentNotification,
    ) -> anyhow::Result<()>;

    /// Cancels all pending notifications for an incident
    /// This is called when the incident is resolved or when it is acknowledged by the user and prevents further escalations
    async fn cancel_all_notifications_for_incident(
        &self,
        tx: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<()>;
}
