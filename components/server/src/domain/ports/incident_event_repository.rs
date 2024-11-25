use uuid::Uuid;

use crate::domain::entities::incident_event::IncidentEvent;

use super::transactional_repository::TransactionalRepository;

#[async_trait::async_trait]
pub trait IncidentEventRepository: TransactionalRepository + Clone + Send + Sync + 'static {
    async fn create_incident_event(
        &self,
        tx: &mut Self::Transaction,
        event: IncidentEvent,
    ) -> anyhow::Result<()>;

    async fn get_incident_timeline(
        &self,
        organization_id: Uuid,
        incident_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<Vec<IncidentEvent>>;
}
