use uuid::Uuid;

use crate::domain::entities::incident::IncidentWithSourcesDetails;

use super::transactional_repository::TransactionalRepository;

#[async_trait::async_trait]
pub trait IncidentNotificationRepository:
    TransactionalRepository + Clone + Send + Sync + 'static
{
    /// Returns a list of newly-created incidents for which the creation notification has not yet been sent to userrs
    /// This must be executed inside a transaction. Concurrent transactions will not return the same incidents (incidents that are locked by a transaction will be skipped)
    async fn list_new_incidents_due_for_notification(
        &self,
        tx: &mut Self::Transaction,
        limit: u32,
    ) -> anyhow::Result<Vec<IncidentWithSourcesDetails>>;

    /// Stores the fact that the creation notification has been sent for an incident
    async fn acknowledge_incident_creation_notification(
        &self,
        tx: &mut Self::Transaction,
        orgnanization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<()>;
}
