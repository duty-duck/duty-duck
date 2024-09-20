use axum::async_trait;
use uuid::Uuid;

use crate::domain::entities::incident::{
    IncidentPriority, IncidentSource, IncidentStatus, IncidentWithSources, NewIncident,
};

use super::transactional_repository::TransactionalRepository;

#[async_trait]
pub trait IncidentRepository: TransactionalRepository + Clone + Send + Sync + 'static {
    async fn create_incident(
        &self,
        transaction: &mut Self::Transaction,
        incident: NewIncident,
    ) -> anyhow::Result<Uuid>;

    /// Lists incidents for the given organization.
    ///
    /// This function retrieves incidents based on the specified parameters and returns a structured output containing the incidents, total number of incidents, and total number of filtered incidents.
    /// Warning: be careful when using include_priorities and include_statuses. If you want to retrieve all incidents, pass `&IncidentStatus::ALL` and `&IncidentPriority::ALL`.
    /// If you pass `&[]`, no incident will be returned.
    ///
    /// # Arguments
    ///
    /// * `transaction` - A mutable reference to the transaction object.
    /// * `organization_id` - The ID of the organization to filter incidents by.
    /// * `include_statuses` - A slice of `IncidentStatus` values to include in the results. Make sure to include every status you are interested in. Otherwise, the query will return an empty list.
    /// * `include_priorities` - A slice of `IncidentPriority` values to include in the results. Make sure to include every priority you are interested in. Otherwise, the query will return an empty list.
    /// * `include_sources` - A slice of `IncidentSource` values to include in the results.
    /// * `limit` - The maximum number of incidents to return.
    /// * `offset` - The number of incidents to skip before returning the results.
    ///
    /// # Returns
    ///
    /// A `ListIncidentsOutput` struct containing the incidents, total number of incidents, and total number of filtered incidents.
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

pub struct ListIncidentsOutput {
    pub incidents: Vec<IncidentWithSources>,
    pub total_incidents: u32,
    pub total_filtered_incidents: u32,
}
