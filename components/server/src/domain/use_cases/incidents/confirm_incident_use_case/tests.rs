use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    entities::{
        entity_metadata::EntityMetadata,
        http_monitor::HttpMonitorErrorKind,
        incident::{
            HttpMonitorIncidentCause, HttpMonitorIncidentCausePing, Incident, IncidentCause, IncidentPriority, IncidentStatus
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

use super::{confirm_incident, NotificationOpts};

fn create_test_incident(org_id: Uuid, status: IncidentStatus) -> Incident {
    Incident {
        id: Uuid::new_v4(),
        organization_id: org_id,
        created_at: Utc::now(),
        created_by: Some(Uuid::new_v4()),
        resolved_at: None,
        cause: Some(IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
            last_ping: HttpMonitorIncidentCausePing {
                error_kind: HttpMonitorErrorKind::Timeout,
                http_code: None,
            },
            previous_pings: vec![],
        })),
        status,
        priority: IncidentPriority::Critical,
        incident_source_type: crate::domain::entities::incident::IncidentSourceType::HttpMonitor,
        incident_source_id: Uuid::new_v4(),
        acknowledged_by: vec![],
        metadata: EntityMetadata::default(),
    }
}

#[tokio::test]
async fn test_confirm_incident_success() -> anyhow::Result<()> {
    // Setup repositories
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    // Create test incident
    let org_id = Uuid::new_v4();
    let incident = create_test_incident(org_id, IncidentStatus::ToBeConfirmed);

    // Add incident to repository
    {
        let mut state = incident_repo.state.lock().await;
        state.push(incident.clone());
    }

    // Execute use case
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

    let mut tx = incident_repo.begin_transaction().await?;
    confirm_incident(
        &mut tx,
        &incident_repo,
        &incident_event_repo,
        &incident_notification_repo,
        &incident,
        notification_opts.clone(),
    )
    .await?;

    // Verify incident status was updated
    let state = incident_repo.state.lock().await;
    let updated_incident = state.iter().next().expect("Incident should exist");
    assert_eq!(updated_incident.status, IncidentStatus::Ongoing);

    // Verify event was created
    let event_state = incident_event_repo.state.lock().await;
    let event = event_state.iter().next().expect("Event should exist");
    assert_eq!(event.event_type, IncidentEventType::Confirmation);
    assert_eq!(event.incident_id, incident.id);
    assert_eq!(event.organization_id, org_id);

    // Verify notification was created
    let notification_state = incident_notification_repo.state.lock().await;
    let notification = notification_state
        .iter()
        .next()
        .expect("Notification should exist");
    assert_eq!(notification.incident_id, incident.id);
    assert_eq!(notification.organization_id, org_id);
    assert_eq!(
        notification.notification_type,
        IncidentNotificationType::IncidentConfirmation
    );
    assert_eq!(
        notification.notification_payload,
        notification_opts.notification_payload
    );
    assert!(notification.send_sms);
    assert!(notification.send_push_notification);
    assert!(notification.send_email);

    Ok(())
}

#[tokio::test]
async fn test_confirm_incident_wrong_status() -> anyhow::Result<()> {
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let incident = create_test_incident(org_id, IncidentStatus::Ongoing);

    // Add incident to repository
    {
        let mut state = incident_repo.state.lock().await;
        state.push(incident.clone());
    }

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

    // Execute use case
    let mut tx = incident_repo.begin_transaction().await?;
    let result = confirm_incident(
        &mut tx,
        &incident_repo,
        &incident_event_repo,
        &incident_notification_repo,
        &incident,
        notification_opts,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Incident is not to be confirmed"
    );

    // Verify no changes were made
    let event_state = incident_event_repo.state.lock().await;
    assert!(event_state.is_empty());

    let notification_state = incident_notification_repo.state.lock().await;
    assert!(notification_state.is_empty());

    Ok(())
}
