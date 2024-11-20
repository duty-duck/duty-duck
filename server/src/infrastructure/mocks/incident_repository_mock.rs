use axum::async_trait;
use chrono::Utc;
use std::sync::{Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::entities::entity_metadata::FilterableMetadata;
use crate::domain::entities::incident::{
    Incident, IncidentSource, IncidentSourceType, NewIncident,
};
use crate::domain::ports::incident_repository::{
    IncidentRepository, ListIncidentsOpts, ListIncidentsOutput,
};
use crate::domain::ports::transactional_repository::{TransactionMock, TransactionalRepository};

#[derive(Clone)]
pub struct IncidentRepositoryMock {
    pub state: Arc<Mutex<Vec<Incident>>>,
}

impl IncidentRepositoryMock {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl TransactionalRepository for IncidentRepositoryMock {
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
impl IncidentRepository for IncidentRepositoryMock {
    async fn get_incident(
        &self,
        _transaction: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<Option<Incident>> {
        let state = self.state.lock().await;
        Ok(state
            .iter()
            .find(|i| i.id == incident_id && i.organization_id == organization_id)
            .cloned())
    }

    async fn create_incident(
        &self,
        _transaction: &mut Self::Transaction,
        incident: NewIncident,
    ) -> anyhow::Result<Uuid> {
        let id = Uuid::new_v4();
        let (incident_source_type, incident_source_id) = match incident.source {
            IncidentSource::HttpMonitor { id } => (IncidentSourceType::HttpMonitor, id),
        };
        let incident = Incident {
            organization_id: incident.organization_id,
            id,
            created_at: Utc::now(),
            created_by: incident.created_by,
            resolved_at: None,
            cause: incident.cause,
            status: incident.status,
            priority: incident.priority,
            incident_source_type,
            incident_source_id,
            acknowledged_by: vec![],
            metadata: incident.metadata,
        };

        let mut state = self.state.lock().await;
        state.push(incident);
        Ok(id)
    }

    async fn list_incidents<'a>(
        &self,
        _transaction: &mut Self::Transaction,
        organization_id: Uuid,
        opts: ListIncidentsOpts<'a>,
    ) -> anyhow::Result<ListIncidentsOutput> {
        let state = self.state.lock().await;

        let include_http_monitors_ids = opts
            .include_sources
            .iter()
            .filter_map(|s| match s {
                IncidentSource::HttpMonitor { id } => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();

        let filtered_incidents: Vec<Incident> = state
            .iter()
            .filter(|i| i.organization_id == organization_id)
            .filter(|i| {
                opts.include_statuses.is_empty() || opts.include_statuses.contains(&i.status)
            })
            .filter(|i| {
                opts.include_priorities.is_empty() || opts.include_priorities.contains(&i.priority)
            })
            .filter(|i| {
                opts.include_sources.is_empty()
                    || (i.incident_source_type == IncidentSourceType::HttpMonitor
                        && include_http_monitors_ids.contains(&i.incident_source_id))
            })
            .filter(|i| {
                opts.from_date
                    .map(|date| i.created_at >= date)
                    .unwrap_or(true)
            })
            .filter(|i| {
                opts.to_date
                    .map(|date| i.created_at <= date)
                    .unwrap_or(true)
            })
            .cloned()
            .collect();

        let total_incidents = state.len() as u32;
        let total_filtered_incidents = filtered_incidents.len() as u32;

        let start = opts.offset as usize;
        let end = (opts.offset + opts.limit) as usize;
        let incidents = filtered_incidents
            [start.min(filtered_incidents.len())..end.min(filtered_incidents.len())]
            .to_vec();

        Ok(ListIncidentsOutput {
            incidents,
            total_incidents,
            total_filtered_incidents,
        })
    }

    async fn update_incident(
        &self,
        _transaction: &mut Self::Transaction,
        incident: Incident,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        if let Some(idx) = state.iter().position(|i| i.id == incident.id) {
            state[idx] = incident;
        }
        Ok(())
    }

    async fn acknowledge_incident(
        &self,
        _transaction: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        if let Some(incident) = state
            .iter_mut()
            .find(|i| i.id == incident_id && i.organization_id == organization_id)
        {
            incident.acknowledged_by.push(user_id);
        }
        Ok(())
    }

    async fn delete_incident(
        &self,
        _transaction: &mut Self::Transaction,
        organization_id: Uuid,
        incident_id: Uuid,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        state.retain(|i| !(i.id == incident_id && i.organization_id == organization_id));
        Ok(())
    }

    async fn get_filterable_metadata(
        &self,
        organization_id: Uuid,
    ) -> anyhow::Result<FilterableMetadata> {
        Ok(FilterableMetadata { items: vec![] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::{
            entity_metadata::{EntityMetadata, MetadataFilter},
            http_monitor::HttpMonitorErrorKind,
            incident::{HttpMonitorIncidentCause, HttpMonitorIncidentCausePing, IncidentCause, IncidentPriority, IncidentStatus},
        },
        use_cases::{incidents::OrderIncidentsBy, shared::OrderDirection},
    };

    fn create_test_incident(org_id: Uuid) -> NewIncident {
        NewIncident {
            organization_id: org_id,
            created_by: Some(Uuid::new_v4()),
            cause: Some(IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
                last_ping: HttpMonitorIncidentCausePing {
                    error_kind: HttpMonitorErrorKind::HttpCode,
                    http_code: Some(500),
                },
                previous_pings: vec![],
            })),
            status: IncidentStatus::Ongoing,
            priority: IncidentPriority::Critical,
            source: IncidentSource::HttpMonitor { id: Uuid::new_v4() },
            metadata: EntityMetadata::default(),
        }
    }

    #[tokio::test]
    async fn test_create_incident_updates_state() -> anyhow::Result<()> {
        let repo = IncidentRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let incident = create_test_incident(org_id);
        let source_id = match incident.source {
            IncidentSource::HttpMonitor { id } => id,
        };

        let mut tx = repo.begin_transaction().await?;
        let id = repo.create_incident(&mut tx, incident).await?;

        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);

        let created_incident = &state[0];
        assert_eq!(created_incident.id, id);
        assert_eq!(created_incident.organization_id, org_id);
        assert_eq!(
            created_incident.incident_source_type,
            IncidentSourceType::HttpMonitor
        );
        assert_eq!(created_incident.incident_source_id, source_id);
        assert_eq!(created_incident.status, IncidentStatus::Ongoing);
        assert_eq!(created_incident.priority, IncidentPriority::Critical);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_incidents_with_status_filter() -> anyhow::Result<()> {
        let repo = IncidentRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        // Create incidents with different statuses
        let mut incident1 = create_test_incident(org_id);
        incident1.status = IncidentStatus::Ongoing;
        let mut incident2 = create_test_incident(org_id);
        incident2.status = IncidentStatus::Resolved;
        let mut incident3 = create_test_incident(org_id);
        incident3.status = IncidentStatus::ToBeConfirmed;

        repo.create_incident(&mut tx, incident1).await?;
        repo.create_incident(&mut tx, incident2).await?;
        repo.create_incident(&mut tx, incident3).await?;

        let result = repo
            .list_incidents(
                &mut tx,
                org_id,
                ListIncidentsOpts {
                    include_statuses: &[IncidentStatus::Ongoing],
                    include_priorities: &[],
                    include_sources: &[],
                    from_date: None,
                    to_date: None,
                    offset: 0,
                    limit: 10,
                    order_by: OrderIncidentsBy::CreatedAt,
                    order_direction: OrderDirection::Desc,
                    metadata_filter: MetadataFilter::default(),
                },
            )
            .await?;

        assert_eq!(result.incidents.len(), 1);
        assert_eq!(result.incidents[0].status, IncidentStatus::Ongoing);
        assert_eq!(result.total_incidents, 3);
        assert_eq!(result.total_filtered_incidents, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_incidents_with_priority_filter() -> anyhow::Result<()> {
        let repo = IncidentRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        // Create incidents with different priorities
        let mut incident1 = create_test_incident(org_id);
        incident1.priority = IncidentPriority::Critical;
        let mut incident2 = create_test_incident(org_id);
        incident2.priority = IncidentPriority::Major;
        let mut incident3 = create_test_incident(org_id);
        incident3.priority = IncidentPriority::Minor;

        repo.create_incident(&mut tx, incident1).await?;
        repo.create_incident(&mut tx, incident2).await?;
        repo.create_incident(&mut tx, incident3).await?;

        let result = repo
            .list_incidents(
                &mut tx,
                org_id,
                ListIncidentsOpts {
                    include_statuses: &[],
                    include_priorities: &[IncidentPriority::Critical, IncidentPriority::Major],
                    include_sources: &[],
                    from_date: None,
                    to_date: None,
                    offset: 0,
                    limit: 10,
                    order_by: OrderIncidentsBy::CreatedAt,
                    order_direction: OrderDirection::Desc,
                    metadata_filter: MetadataFilter::default(),
                },
            )
            .await?;

        assert_eq!(result.incidents.len(), 2);
        assert_eq!(result.total_incidents, 3);
        assert_eq!(result.total_filtered_incidents, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_incidents_with_date_filter() -> anyhow::Result<()> {
        let repo = IncidentRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        // Create three incidents
        for _ in 0..3 {
            repo.create_incident(&mut tx, create_test_incident(org_id))
                .await?;
        }

        let now = Utc::now();
        let result = repo
            .list_incidents(
                &mut tx,
                org_id,
                ListIncidentsOpts {
                    include_statuses: &[],
                    include_priorities: &[],
                    include_sources: &[],
                    from_date: Some(now - chrono::Duration::hours(1)),
                    to_date: Some(now + chrono::Duration::hours(1)),
                    offset: 0,
                    limit: 10,
                    order_by: OrderIncidentsBy::CreatedAt,
                    order_direction: OrderDirection::Desc,
                    metadata_filter: MetadataFilter::default(),
                },
            )
            .await?;

        assert_eq!(result.incidents.len(), 3);
        for incident in result.incidents {
            assert!(incident.created_at >= now - chrono::Duration::hours(1));
            assert!(incident.created_at <= now + chrono::Duration::hours(1));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_acknowledge_incident() -> anyhow::Result<()> {
        let repo = IncidentRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        let incident = create_test_incident(org_id);
        let incident_id = repo.create_incident(&mut tx, incident).await?;

        repo.acknowledge_incident(&mut tx, org_id, incident_id, user_id)
            .await?;

        let state = repo.state.lock().await;
        let incident = state
            .iter()
            .find(|i| i.id == incident_id)
            .expect("Incident should exist");
        assert!(incident.acknowledged_by.contains(&user_id));

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_incident() -> anyhow::Result<()> {
        let repo = IncidentRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        let incident = create_test_incident(org_id);
        let incident_id = repo.create_incident(&mut tx, incident).await?;

        // Verify incident exists
        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);
        drop(state);

        // Delete incident
        repo.delete_incident(&mut tx, org_id, incident_id).await?;

        // Verify incident was deleted
        let state = repo.state.lock().await;
        assert_eq!(state.len(), 0);

        Ok(())
    }
}
