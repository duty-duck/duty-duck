use axum::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;


use crate::domain::{entities::incident::{
    Incident, IncidentPriority, IncidentSource, IncidentStatus, NewIncident,
}, use_cases::{incidents::OrderIncidentsBy, shared::OrderDirection}};

use super::transactional_repository::TransactionalRepository;

#[async_trait]
pub trait IncidentRepository: TransactionalRepository + Clone + Send + Sync + 'static {
    /// Gets the incident with the given ID.
    ///
    /// # Arguments
    ///
    /// * `organization_id` - The ID of the organization to get the incident for.
    /// * `incident_id` - The ID of the incident to get.
    /// * `transaction` - A mutable reference to the transaction object.
    ///
    /// # Returns
    ///
    /// An `Option<Incident>` containing the incident if it exists, or `None` if it does not.
    async fn get_incident(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<Option<Incident>>;

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
    /// * `from_date` - The start date to filter incidents by.
    /// * `to_date` - The end date to filter incidents by.
    ///
    /// # Returns
    ///
    /// A `ListIncidentsOutput` struct containing the incidents, total number of incidents, and total number of filtered incidents.
    #[allow(clippy::too_many_arguments)]
    async fn list_incidents<'a>(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        opts: ListIncidentsOpts<'a>,
    ) -> anyhow::Result<ListIncidentsOutput>;

    /// Updates the incident with the given ID.
    async fn update_incident(
        &self,
        transaction: &mut Self::Transaction,
        incident: Incident,
    ) -> anyhow::Result<()>;

    /// Marks the incident as acknowledged by the given user.
    ///
    /// # Arguments
    ///
    /// * `transaction` - A mutable reference to the transaction object.
    /// * `organization_id` - The ID of the organization to acknowledge incidents for.
    /// * `incident_id` - The ID of the incident to acknowledge.
    /// * `user_id` - The ID of the user acknowledging the incident.
    async fn acknowledge_incident(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<()>;

    /// Deletes an incident with the given ID.
    async fn delete_incident(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<()>;
}

pub struct ListIncidentsOutput {
    pub incidents: Vec<Incident>,
    pub total_incidents: u32,
    pub total_filtered_incidents: u32,
}

#[derive(Clone, Debug, Default)]
pub struct ListIncidentsOpts<'a> {
    pub include_statuses: &'a [IncidentStatus],
    pub include_priorities: &'a [IncidentPriority],
    pub include_sources: &'a [IncidentSource],
    pub limit: u32,
    pub offset: u32,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub order_by: OrderIncidentsBy,
    pub order_direction: OrderDirection,
}