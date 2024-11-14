use uuid::Uuid;

use crate::domain::{
    entities::{
        entity_metadata::EntityMetadata,
        http_monitor::HttpMonitorErrorKind,
        incident::{
            HttpMonitorIncidentCause, HttpMonitorIncidentCausePing, IncidentCause, IncidentPriority, IncidentSource, IncidentStatus, NewIncident
        },
        incident_event::IncidentEventType,
        incident_notification::{IncidentNotificationPayload, IncidentNotificationType},
    },
    ports::transactional_repository::TransactionalRepository,
};
use crate::infrastructure::mocks::{
    incident_event_repository_mock::IncidentEventRepositoryMock,
    incident_notification_repository_mock::IncidentNotificationRepositoryMock,
    incident_repository_mock::IncidentRepositoryMock,
};

use super::{create_incident, NotificationOpts};

fn create_test_new_incident(org_id: Uuid) -> NewIncident {
    NewIncident {
        organization_id: org_id,
        created_by: Some(Uuid::new_v4()),
        status: IncidentStatus::Ongoing,
        priority: IncidentPriority::Critical,
        source: IncidentSource::HttpMonitor { id: Uuid::new_v4() },
        cause: Some(IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
            last_ping: HttpMonitorIncidentCausePing {
                error_kind: HttpMonitorErrorKind::Timeout,
                http_code: None,
            },
            previous_pings: vec![],
        })),
        metadata: EntityMetadata::default(),
    }
}

#[tokio::test]
async fn test_create_incident_success() -> anyhow::Result<()> {
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let new_incident = create_test_new_incident(org_id);
    let mut tx = incident_repo.begin_transaction().await?;

    let notification_opts = NotificationOpts {
        send_sms: true,
        send_push_notification: true,
        send_email: true,
        notification_payload: IncidentNotificationPayload {
            incident_cause: IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
                last_ping: HttpMonitorIncidentCausePing {
                    error_kind: HttpMonitorErrorKind::Timeout,
                    http_code: None,
                },
                previous_pings: vec![],
            }),
            incident_http_monitor_url: None,
        },
    };

    let incident_id = create_incident(
        &mut tx,
        &incident_repo,
        &incident_event_repo,
        &incident_notification_repo,
        new_incident.clone(),
        Some(notification_opts),
    )
    .await?;

    // Verify incident was created
    let incident_state = incident_repo.state.lock().await;
    assert_eq!(incident_state.len(), 1);
    let created_incident = &incident_state[0];
    assert_eq!(created_incident.id, incident_id);
    assert_eq!(created_incident.organization_id, org_id);
    assert_eq!(created_incident.status, IncidentStatus::Ongoing);

    // Verify event was created
    let event_state = incident_event_repo.state.lock().await;
    assert_eq!(event_state.len(), 1);
    let event = &event_state[0];
    assert_eq!(event.incident_id, incident_id);
    assert_eq!(event.event_type, IncidentEventType::Creation);

    // Verify notification was created
    let notification_state = incident_notification_repo.state.lock().await;
    assert_eq!(notification_state.len(), 1);
    let notification = &notification_state[0];
    assert_eq!(notification.incident_id, incident_id);
    assert_eq!(notification.organization_id, org_id);
    assert_eq!(
        notification.notification_type,
        IncidentNotificationType::IncidentCreation
    );
    assert!(matches!(
        notification.notification_payload,
        IncidentNotificationPayload {
            incident_cause: IncidentCause::HttpMonitorIncidentCause { .. },
            incident_http_monitor_url: None,
        }
    ));

    Ok(())
}

#[tokio::test]
async fn test_create_incident_without_notifications() -> anyhow::Result<()> {
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let new_incident = create_test_new_incident(org_id);
    let mut tx = incident_repo.begin_transaction().await?;

    let incident_id = create_incident(
        &mut tx,
        &incident_repo,
        &incident_event_repo,
        &incident_notification_repo,
        new_incident.clone(),
        None,
    )
    .await?;

    // Verify incident was created
    let incident_state = incident_repo.state.lock().await;
    assert_eq!(incident_state.len(), 1);

    // Verify event was created
    let event_state = incident_event_repo.state.lock().await;
    assert_eq!(event_state.len(), 1);

    // Verify no notification was created
    let notification_state = incident_notification_repo.state.lock().await;
    assert!(notification_state.is_empty());

    Ok(())
}