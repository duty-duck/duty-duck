use axum::async_trait;
use uuid::Uuid;

use crate::domain::entities::incident::{
    IncidentCause, IncidentPriority, IncidentSource, IncidentStatus, IncidentWithSources
};

use super::transactional_repository::TransactionalRepository;

#[async_trait]
pub trait IncidentRepository: TransactionalRepository + Clone + Send + Sync + 'static {
    async fn create_incident(
        &self,
        transaction: &mut Self::Transaction,
        incident: NewIncident,
    ) -> anyhow::Result<Uuid>;

    #[allow(clippy::too_many_arguments)]
    async fn list_incidents(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        include_statuses: &[IncidentStatus],
        include_priorities: &[IncidentPriority],
        include_sources: &[IncidentSource],
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<ListIncidentsOutput>;

    async fn resolve_incidents_by_source(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        sources: &[IncidentSource],
    ) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct NewIncident {
    pub organization_id: Uuid,
    pub created_by: Option<Uuid>,
    pub status: IncidentStatus,
    pub priority: IncidentPriority,
    pub sources: Vec<IncidentSource>,
    pub cause: Option<IncidentCause>
}

pub struct ListIncidentsOutput {
    pub incidents: Vec<IncidentWithSources>,
    pub total_incidents: u32,
    pub total_filtered_incidents: u32,
}
