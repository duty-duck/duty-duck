use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::AuthContext,
        entity_metadata::EntityMetadata,
        http_monitor::HttpMonitorErrorKind,
        incident::{
            HttpMonitorIncidentCause, HttpMonitorIncidentCausePing, Incident, IncidentCause,
            IncidentPriority, IncidentSourceType, IncidentStatus,
        },
        incident_event::IncidentEventType,
        organization::OrganizationUserRole,
    },
    ports::transactional_repository::TransactionalRepository,
};
use crate::infrastructure::mocks::{
    incident_event_repository_mock::IncidentEventRepositoryMock,
    incident_notification_repository_mock::IncidentNotificationRepositoryMock,
    incident_repository_mock::IncidentRepositoryMock,
};

use super::{acknowledge_incident, AcknowledgeIncidentError};

fn create_test_incident(org_id: Uuid) -> Incident {
    Incident {
        id: Uuid::new_v4(),
        organization_id: org_id,
        created_at: Utc::now(),
        created_by: Some(Uuid::new_v4()),
        resolved_at: None,
        cause: Some(IncidentCause::HttpMonitorIncidentCause(
            HttpMonitorIncidentCause {
                last_ping: HttpMonitorIncidentCausePing {
                    error_kind: HttpMonitorErrorKind::Timeout,
                    http_code: None,
                },
                previous_pings: vec![],
            },
        )),
        status: IncidentStatus::Ongoing,
        priority: IncidentPriority::Critical,
        incident_source_type: IncidentSourceType::HttpMonitor,
        incident_source_id: Uuid::new_v4(),
        acknowledged_by: vec![],
        metadata: EntityMetadata::default(),
    }
}

#[tokio::test]
async fn test_acknowledge_incident_success() -> anyhow::Result<()> {
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let auth_context =
        AuthContext::test_context(org_id, user_id, &[OrganizationUserRole::Administrator], &[]);

    // Create and store test incident
    let incident = create_test_incident(org_id);
    let _tx = incident_repo.begin_transaction().await?;
    {
        let mut state = incident_repo.state.lock().await;
        state.push(incident.clone());
    }

    // Execute use case
    acknowledge_incident(
        &auth_context,
        &incident_repo,
        &incident_event_repo,
        &incident_notification_repo,
        incident.id,
    )
    .await?;

    // Verify incident was acknowledged
    let state = incident_repo.state.lock().await;
    let updated_incident = state.iter().next().expect("Incident should exist");
    assert!(updated_incident.acknowledged_by.contains(&user_id));

    // Verify event was created
    let event_state = incident_event_repo.state.lock().await;
    let event = event_state.iter().next().expect("Event should exist");
    assert_eq!(event.event_type, IncidentEventType::Acknowledged);
    assert_eq!(event.incident_id, incident.id);
    assert_eq!(event.organization_id, org_id);
    assert_eq!(event.user_id, Some(user_id));

    // Verify notifications were cancelled
    let notification_state = incident_notification_repo.state.lock().await;
    assert!(notification_state.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_acknowledge_incident_already_acknowledged() -> anyhow::Result<()> {
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let auth_context =
        AuthContext::test_context(org_id, user_id, &[OrganizationUserRole::Administrator], &[]);

    // Create incident that's already acknowledged by the user
    let mut incident = create_test_incident(org_id);
    incident.acknowledged_by.push(user_id);

    // Add incident to repository
    let _tx = incident_repo.begin_transaction().await?;
    {
        let mut state = incident_repo.state.lock().await;
        state.push(incident.clone());
    }

    // Execute use case
    acknowledge_incident(
        &auth_context,
        &incident_repo,
        &incident_event_repo,
        &incident_notification_repo,
        incident.id,
    )
    .await?;

    // Verify no event was created
    let event_state = incident_event_repo.state.lock().await;
    assert!(event_state.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_acknowledge_incident_forbidden() -> anyhow::Result<()> {
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let auth_context =
        AuthContext::test_context(org_id, user_id, &[OrganizationUserRole::Reporter], &[]);

    let incident = create_test_incident(org_id);
    let _tx = incident_repo.begin_transaction().await?;
    {
        let mut state = incident_repo.state.lock().await;
        state.push(incident.clone());
    }

    // Execute use case
    let result = acknowledge_incident(
        &auth_context,
        &incident_repo,
        &incident_event_repo,
        &incident_notification_repo,
        incident.id,
    )
    .await;

    assert!(matches!(result, Err(AcknowledgeIncidentError::Forbidden)));

    // Verify no changes were made
    let event_state = incident_event_repo.state.lock().await;
    assert!(event_state.is_empty());

    let incident_state = incident_repo.state.lock().await;
    let unchanged_incident = incident_state.iter().next().expect("Incident should exist");
    assert!(unchanged_incident.acknowledged_by.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_acknowledge_incident_not_found() -> anyhow::Result<()> {
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let auth_context =
        AuthContext::test_context(org_id, user_id, &[OrganizationUserRole::Administrator], &[]);

    // Execute use case with non-existent incident ID
    let result = acknowledge_incident(
        &auth_context,
        &incident_repo,
        &incident_event_repo,
        &incident_notification_repo,
        Uuid::new_v4(),
    )
    .await;

    assert!(matches!(
        result,
        Err(AcknowledgeIncidentError::IncidentNotFound)
    ));

    // Verify no changes were made
    let event_state = incident_event_repo.state.lock().await;
    assert!(event_state.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_acknowledge_incident_wrong_organization() -> anyhow::Result<()> {
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let other_org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let auth_context = AuthContext::test_context(
        other_org_id,
        user_id,
        &[OrganizationUserRole::Administrator],
        &[],
    );

    // Create incident in different organization
    let incident = create_test_incident(org_id);
    let _tx = incident_repo.begin_transaction().await?;
    {
        let mut state = incident_repo.state.lock().await;
        state.push(incident.clone());
    }

    // Execute use case
    let result = acknowledge_incident(
        &auth_context,
        &incident_repo,
        &incident_event_repo,
        &incident_notification_repo,
        incident.id,
    )
    .await;

    assert!(matches!(
        result,
        Err(AcknowledgeIncidentError::IncidentNotFound)
    ));

    // Verify no changes were made
    let event_state = incident_event_repo.state.lock().await;
    assert!(event_state.is_empty());

    let incident_state = incident_repo.state.lock().await;
    let unchanged_incident = incident_state.iter().next().expect("Incident should exist");
    assert!(unchanged_incident.acknowledged_by.is_empty());

    Ok(())
}
