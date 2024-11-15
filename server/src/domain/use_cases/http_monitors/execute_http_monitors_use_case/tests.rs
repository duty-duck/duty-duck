use chrono::Utc;
use uuid::Uuid;

use crate::domain::entities::http_monitor::RequestHeaders;
use crate::domain::entities::incident::HttpMonitorIncidentCause;
use crate::infrastructure::mocks::file_storage_mock::FileStorageMock;
use crate::infrastructure::mocks::{
    http_monitor_repository_mock::HttpMonitorRepositoryMock,
    incident_event_repository_mock::IncidentEventRepositoryMock,
    incident_notification_repository_mock::IncidentNotificationRepositoryMock,
    incident_repository_mock::IncidentRepositoryMock,
};
use crate::{
    domain::{
        entities::{
            entity_metadata::EntityMetadata,
            http_monitor::{HttpMonitor, HttpMonitorErrorKind, HttpMonitorStatus},
            incident::{
                HttpMonitorIncidentCausePing, Incident, IncidentCause, IncidentPriority,
                IncidentSourceType, IncidentStatus,
            },
            incident_event::{IncidentEventPayload, IncidentEventType},
        },
        ports::{http_client::PingResponse, transactional_repository::TransactionalRepository},
    },
    infrastructure::mocks::http_client_mock::HttpClientMock,
};

use super::ExecuteHttpMonitorsUseCase;

fn create_test_monitor(org_id: Uuid, status: HttpMonitorStatus) -> HttpMonitor {
    HttpMonitor {
        id: Uuid::new_v4(),
        organization_id: org_id,
        created_at: Utc::now(),
        url: "https://example.com".to_string(),
        first_ping_at: None,
        next_ping_at: Some(Utc::now()),
        last_ping_at: None,
        last_status_change_at: Utc::now(),
        recovery_confirmation_threshold: 3,
        downtime_confirmation_threshold: 3,
        interval_seconds: 60,
        last_http_code: None,
        status,
        status_counter: 0,
        error_kind: HttpMonitorErrorKind::None,
        metadata: EntityMetadata::default(),
        email_notification_enabled: true,
        push_notification_enabled: true,
        sms_notification_enabled: false,
        archived_at: None,
        request_headers: RequestHeaders::default(),
        request_timeout_ms: 2000,
    }
}

fn create_test_incident(org_id: Uuid, monitor: &HttpMonitor, status: IncidentStatus) -> Incident {
    Incident {
        id: Uuid::new_v4(),
        organization_id: org_id,
        created_at: Utc::now(),
        created_by: None,
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
        status,
        priority: IncidentPriority::Critical,
        incident_source_type: IncidentSourceType::HttpMonitor,
        incident_source_id: monitor.id,
        acknowledged_by: vec![],
        metadata: EntityMetadata::default(),
    }
}

#[tokio::test]
async fn test_handle_ping_response_up_to_suspicious() -> anyhow::Result<()> {
    let http_monitor_repo = HttpMonitorRepositoryMock::new();
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let monitor = create_test_monitor(org_id, HttpMonitorStatus::Up);

    let mut tx = http_monitor_repo.begin_transaction().await?;

    // Add monitor to repository
    {
        let mut state = http_monitor_repo.state.lock().await;
        state.push(monitor.clone());
    }

    let use_case = ExecuteHttpMonitorsUseCase {
        http_monitor_repository: http_monitor_repo,
        incident_repository: incident_repo,
        incident_event_repository: incident_event_repo,
        incident_notification_repository: incident_notification_repo,
        http_client: HttpClientMock::new(),
        file_storage: FileStorageMock,
    };

    let ping_response = PingResponse {
        error_kind: HttpMonitorErrorKind::Timeout,
        http_code: None,
        http_headers: Default::default(),
        response_time: std::time::Duration::from_secs(1),
        response_ip_address: None,
        resolved_ip_addresses: vec![],
        response_body_size_bytes: 0,
        response_body_content: None,
        screenshot: None,
    };

    use_case
        .handle_ping_response(&mut tx, monitor, ping_response, None)
        .await?;

    // Verify monitor status changed to Suspicious
    let monitor_state = use_case.http_monitor_repository.state.lock().await;
    let updated_monitor = monitor_state.first().expect("Monitor should exist");
    assert_eq!(updated_monitor.status, HttpMonitorStatus::Suspicious);
    assert_eq!(updated_monitor.status_counter, 1);
    assert_eq!(updated_monitor.error_kind, HttpMonitorErrorKind::Timeout);

    // Verify an unconfirmed incident was created
    let incident_state = use_case.incident_repository.state.lock().await;
    let incident = incident_state.first().expect("Incident should exist");
    assert_eq!(incident.status, IncidentStatus::ToBeConfirmed);

    Ok(())
}

#[tokio::test]
async fn test_handle_ping_response_down_without_existing_incident() -> anyhow::Result<()> {
    let http_monitor_repo = HttpMonitorRepositoryMock::new();
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();

    // Create a monitor with an unknown status (like a newly created monitor)
    let mut monitor = create_test_monitor(org_id, HttpMonitorStatus::Unknown);
    // incidents are confirmed automatically after one downtime
    monitor.downtime_confirmation_threshold = 1;

    let mut tx = http_monitor_repo.begin_transaction().await?;

    // Add monitor to repositories
    {
        let mut monitor_state = http_monitor_repo.state.lock().await;
        monitor_state.push(monitor.clone());
    }

    let use_case = ExecuteHttpMonitorsUseCase {
        http_monitor_repository: http_monitor_repo,
        incident_repository: incident_repo,
        incident_event_repository: incident_event_repo,
        incident_notification_repository: incident_notification_repo,
        http_client: HttpClientMock::new(),
        file_storage: FileStorageMock,
    };

    let ping_response = PingResponse {
        error_kind: HttpMonitorErrorKind::Timeout,
        http_code: None,
        http_headers: Default::default(),
        response_time: std::time::Duration::from_secs(1),
        response_ip_address: None,
        resolved_ip_addresses: vec![],
        response_body_size_bytes: 0,
        response_body_content: None,
        screenshot: None,
    };

    // execute the use case
    use_case
        .handle_ping_response(&mut tx, monitor, ping_response, None)
        .await?;

    // Verify monitor status changed to Down
    let monitor_state = use_case.http_monitor_repository.state.lock().await;
    let updated_monitor = monitor_state.first().expect("Monitor should exist");
    assert_eq!(updated_monitor.status, HttpMonitorStatus::Down);
    assert_eq!(updated_monitor.status_counter, 1);

    // Verify an ongoing incident was created
    let incident_state = use_case.incident_repository.state.lock().await;
    let incident = incident_state.first().expect("Incident should exist");
    assert_eq!(incident.status, IncidentStatus::Ongoing);

    Ok(())
}

#[tokio::test]
async fn test_handle_ping_response_suspicious_to_down() -> anyhow::Result<()> {
    let http_monitor_repo = HttpMonitorRepositoryMock::new();
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let mut monitor = create_test_monitor(org_id, HttpMonitorStatus::Suspicious);
    monitor.status_counter = 2;

    let mut tx = http_monitor_repo.begin_transaction().await?;

    // Add monitor and existing incident to repositories
    {
        let mut monitor_state = http_monitor_repo.state.lock().await;
        monitor_state.push(monitor.clone());

        let mut incident_state = incident_repo.state.lock().await;
        incident_state.push(create_test_incident(
            org_id,
            &monitor,
            IncidentStatus::ToBeConfirmed,
        ));
    }

    let use_case = ExecuteHttpMonitorsUseCase {
        http_monitor_repository: http_monitor_repo,
        incident_repository: incident_repo,
        incident_event_repository: incident_event_repo,
        incident_notification_repository: incident_notification_repo,
        http_client: HttpClientMock::new(),
        file_storage: FileStorageMock,
    };

    let ping_response = PingResponse {
        error_kind: HttpMonitorErrorKind::Timeout,
        http_code: None,
        http_headers: Default::default(),
        response_time: std::time::Duration::from_secs(1),
        response_ip_address: None,
        resolved_ip_addresses: vec![],
        response_body_size_bytes: 0,
        response_body_content: None,
        screenshot: None,
    };

    let existing_incident = use_case
        .incident_repository
        .state
        .lock()
        .await
        .first()
        .cloned();
    use_case
        .handle_ping_response(&mut tx, monitor, ping_response, existing_incident)
        .await?;

    // Verify monitor status changed to Down
    let monitor_state = use_case.http_monitor_repository.state.lock().await;
    let updated_monitor = monitor_state.first().expect("Monitor should exist");
    assert_eq!(updated_monitor.status, HttpMonitorStatus::Down);

    // status counter is reset to one because the status changed from Suspicious to Down
    assert_eq!(updated_monitor.status_counter, 1);

    // Verify incident was confirmed
    let incident_state = use_case.incident_repository.state.lock().await;
    let incident = incident_state.first().expect("Incident should exist");
    assert_eq!(incident.status, IncidentStatus::Ongoing);

    Ok(())
}

#[tokio::test]
async fn test_handle_ping_response_down_to_recovering() -> anyhow::Result<()> {
    let http_monitor_repo = HttpMonitorRepositoryMock::new();
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let mut monitor = create_test_monitor(org_id, HttpMonitorStatus::Down);
    monitor.status_counter = 3;

    let mut tx = http_monitor_repo.begin_transaction().await?;

    // Add monitor and existing incident to repositories
    {
        let mut monitor_state = http_monitor_repo.state.lock().await;
        monitor_state.push(monitor.clone());

        let mut incident_state = incident_repo.state.lock().await;
        incident_state.push(create_test_incident(
            org_id,
            &monitor,
            IncidentStatus::Ongoing,
        ));
    }

    let use_case = ExecuteHttpMonitorsUseCase {
        http_monitor_repository: http_monitor_repo,
        incident_repository: incident_repo,
        incident_event_repository: incident_event_repo,
        incident_notification_repository: incident_notification_repo,
        http_client: HttpClientMock::new(),
        file_storage: FileStorageMock,
    };

    let ping_response = PingResponse {
        error_kind: HttpMonitorErrorKind::None,
        http_code: Some(200),
        http_headers: Default::default(),
        response_time: std::time::Duration::from_secs(1),
        response_ip_address: None,
        resolved_ip_addresses: vec![],
        response_body_size_bytes: 0,
        response_body_content: None,
        screenshot: None,
    };

    let existing_incident = use_case
        .incident_repository
        .state
        .lock()
        .await
        .first()
        .cloned();
    use_case
        .handle_ping_response(&mut tx, monitor, ping_response, existing_incident)
        .await?;

    // Verify monitor status changed to Recovering
    let monitor_state = use_case.http_monitor_repository.state.lock().await;
    let updated_monitor = monitor_state.first().expect("Monitor should exist");
    assert_eq!(updated_monitor.status, HttpMonitorStatus::Recovering);
    assert_eq!(updated_monitor.status_counter, 1);
    assert_eq!(updated_monitor.error_kind, HttpMonitorErrorKind::None);
    assert_eq!(updated_monitor.last_http_code, Some(200));

    Ok(())
}

#[tokio::test]
async fn test_no_new_incident_created_when_incident_exists() -> anyhow::Result<()> {
    let org_id = Uuid::new_v4();
    let http_monitor_repo = HttpMonitorRepositoryMock::new();
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    // Test with each possible incident status
    for status in [
        IncidentStatus::Ongoing,
        IncidentStatus::ToBeConfirmed,
        // IncidentStatus::Resolved is not possible here, because we query explicitly exclude resolved incidents when querying for existing incidents
        // IncidentStatus::Resolved,
    ] {
        let mut monitor = create_test_monitor(org_id, HttpMonitorStatus::Down);
        monitor.status_counter = 3;

        let mut tx = http_monitor_repo.begin_transaction().await?;

        // Add monitor and existing incident to repositories
        {
            let mut monitor_state = http_monitor_repo.state.lock().await;
            monitor_state.clear();
            monitor_state.push(monitor.clone());

            let mut incident_state = incident_repo.state.lock().await;
            incident_state.clear();
            incident_state.push(create_test_incident(org_id, &monitor, status));
        }

        let use_case = ExecuteHttpMonitorsUseCase {
            http_monitor_repository: http_monitor_repo.clone(),
            incident_repository: incident_repo.clone(),
            incident_event_repository: incident_event_repo.clone(),
            incident_notification_repository: incident_notification_repo.clone(),
            http_client: HttpClientMock::new(),
            file_storage: FileStorageMock,
        };

        let ping_response = PingResponse {
            error_kind: HttpMonitorErrorKind::Timeout,
            http_code: None,
            http_headers: Default::default(),
            response_time: std::time::Duration::from_secs(1),
            response_ip_address: None,
            resolved_ip_addresses: vec![],
            response_body_size_bytes: 0,
            response_body_content: None,
            screenshot: None,
        };

        let existing_incident = use_case
            .incident_repository
            .state
            .lock()
            .await
            .first()
            .cloned();

        use_case
            .handle_ping_response(&mut tx, monitor, ping_response, existing_incident)
            .await?;

        // Verify no new incident was created
        let incident_state = use_case.incident_repository.state.lock().await;
        assert_eq!(
            incident_state.len(),
            1,
            "No new incident should be created when status is {:?}",
            status
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_handle_ping_response_updates_incident_cause_when_error_code_changes(
) -> anyhow::Result<()> {
    let http_monitor_repo = HttpMonitorRepositoryMock::new();
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let mut monitor = create_test_monitor(org_id, HttpMonitorStatus::Down);
    monitor.last_http_code = Some(500);

    let mut incident = create_test_incident(org_id, &monitor, IncidentStatus::Ongoing);
    incident.cause = Some(IncidentCause::HttpMonitorIncidentCause(
        HttpMonitorIncidentCause {
            last_ping: HttpMonitorIncidentCausePing {
                error_kind: HttpMonitorErrorKind::HttpCode,
                http_code: Some(500),
            },
            previous_pings: vec![],
        },
    ));

    let mut tx = http_monitor_repo.begin_transaction().await?;

    // Add monitor and incident to repositories
    {
        let mut monitor_state = http_monitor_repo.state.lock().await;
        monitor_state.clear();
        monitor_state.push(monitor.clone());

        let mut incident_state = incident_repo.state.lock().await;
        incident_state.clear();
        incident_state.push(incident.clone());
    }

    let use_case = ExecuteHttpMonitorsUseCase {
        http_monitor_repository: http_monitor_repo.clone(),
        incident_repository: incident_repo.clone(),
        incident_event_repository: incident_event_repo.clone(),
        incident_notification_repository: incident_notification_repo.clone(),
        http_client: HttpClientMock::new(),
        file_storage: FileStorageMock,
    };

    let ping_response = PingResponse {
        error_kind: HttpMonitorErrorKind::HttpCode,
        http_code: Some(422),
        http_headers: Default::default(),
        response_time: std::time::Duration::from_secs(1),
        response_ip_address: None,
        resolved_ip_addresses: vec![],
        response_body_size_bytes: 0,
        response_body_content: None,
        screenshot: None,
    };

    let existing_incident = use_case
        .incident_repository
        .state
        .lock()
        .await
        .first()
        .cloned();
    use_case
        .handle_ping_response(&mut tx, monitor, ping_response, existing_incident)
        .await?;

    // Verify incident still exists and is ongoing
    let incident_state = use_case.incident_repository.state.lock().await;
    assert_eq!(incident_state.len(), 1, "Incident should still exist");
    let updated_incident = incident_state.first().expect("Incident should exist");
    assert_eq!(updated_incident.status, IncidentStatus::Ongoing);

    // Verify that the previous pings field in the incident cause has been updated
    #[allow(irrefutable_let_patterns)]
    if let IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
        previous_pings,
        ..
    }) = &updated_incident.cause.as_ref().unwrap()
    {
        assert_eq!(previous_pings.len(), 1);
        assert_eq!(
            previous_pings[0],
            HttpMonitorIncidentCausePing {
                error_kind: HttpMonitorErrorKind::HttpCode,
                http_code: Some(500)
            }
        );
    } else {
        panic!("Incident cause should be HttpMonitorIncidentCause");
    }

    // Verify a ping event and a switch to down event was created
    let events = use_case.incident_event_repository.state.lock().await;
    assert_eq!(events.len(), 2, "two events should be created");
    let ping_event = events.first().expect("Event should exist");
    assert_eq!(ping_event.event_type, IncidentEventType::MonitorPinged);

    if let Some(IncidentEventPayload::MonitorPing(payload)) = &ping_event.event_payload {
        assert_eq!(payload.http_code, Some(422));
    } else {
        panic!("Ping event payload should be MonitorPing");
    }

    let switch_to_down_event = events.last().expect("Event should exist");
    assert_eq!(switch_to_down_event.event_type, IncidentEventType::MonitorSwitchedToDown);

    Ok(())
}

#[tokio::test]
async fn test_handle_ping_response_http_code_500_to_recovering() -> anyhow::Result<()> {
    let http_monitor_repo = HttpMonitorRepositoryMock::new();
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let mut monitor = create_test_monitor(org_id, HttpMonitorStatus::Down);
    monitor.status_counter = 3;
    monitor.error_kind = HttpMonitorErrorKind::HttpCode;
    monitor.last_http_code = Some(500);

    let mut tx = http_monitor_repo.begin_transaction().await?;

    // Add monitor and existing incident to repositories
    {
        let mut monitor_state = http_monitor_repo.state.lock().await;
        monitor_state.push(monitor.clone());

        let mut incident_state = incident_repo.state.lock().await;
        let mut incident = create_test_incident(org_id, &monitor, IncidentStatus::Ongoing);
        incident.cause = Some(IncidentCause::HttpMonitorIncidentCause(
            HttpMonitorIncidentCause {
                last_ping: HttpMonitorIncidentCausePing {
                    error_kind: HttpMonitorErrorKind::HttpCode,
                    http_code: Some(500),
                },
                previous_pings: vec![],
            },
        ));
        incident_state.push(incident);
    }

    let use_case = ExecuteHttpMonitorsUseCase {
        http_monitor_repository: http_monitor_repo,
        incident_repository: incident_repo,
        incident_event_repository: incident_event_repo,
        incident_notification_repository: incident_notification_repo,
        http_client: HttpClientMock::new(),
        file_storage: FileStorageMock,
    };

    let ping_response = PingResponse {
        error_kind: HttpMonitorErrorKind::None,
        http_code: Some(200),
        http_headers: Default::default(),
        response_time: std::time::Duration::from_secs(1),
        response_ip_address: None,
        resolved_ip_addresses: vec![],
        response_body_size_bytes: 0,
        response_body_content: None,
        screenshot: None,
    };

    let existing_incident = use_case
        .incident_repository
        .state
        .lock()
        .await
        .first()
        .cloned();
    use_case
        .handle_ping_response(&mut tx, monitor, ping_response, existing_incident)
        .await?;

    // Verify monitor status changed to Recovering
    let monitor_state = use_case.http_monitor_repository.state.lock().await;
    let updated_monitor = monitor_state.first().expect("Monitor should exist");
    assert_eq!(updated_monitor.status, HttpMonitorStatus::Recovering);
    assert_eq!(updated_monitor.status_counter, 1);
    assert_eq!(updated_monitor.error_kind, HttpMonitorErrorKind::None);
    assert_eq!(updated_monitor.last_http_code, Some(200));

    // Verify incident still exists and is ongoing
    let incident_state = use_case.incident_repository.state.lock().await;
    assert_eq!(incident_state.len(), 1, "Incident should still exist");
    let updated_incident = incident_state.first().expect("Incident should exist");
    assert_eq!(updated_incident.status, IncidentStatus::Ongoing);

    // Verify that the content of the last ping in the incident cause
    #[allow(irrefutable_let_patterns)]
    if let IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
        last_ping,
        previous_pings,
    }) = &updated_incident
        .cause
        .as_ref()
        .expect("Incident should have a cause")
    {
        assert_eq!(previous_pings.len(), 0);
        assert_eq!(
            last_ping,
            &HttpMonitorIncidentCausePing {
                error_kind: HttpMonitorErrorKind::HttpCode,
                http_code: Some(500)
            }
        );
    } else {
        panic!("Incident cause should be HttpMonitorIncidentCause");
    }

    // Verify a ping event was created and a switch to recovering event
    let events = use_case.incident_event_repository.state.lock().await;
    assert_eq!(events.len(), 2, "Two events should be created");
    let ping_event = events.first().expect("Ping event should exist");
    assert_eq!(ping_event.event_type, IncidentEventType::MonitorPinged);

    if let Some(IncidentEventPayload::MonitorPing(payload)) = &ping_event.event_payload {
        assert_eq!(payload.http_code, Some(200));
        assert_eq!(payload.error_kind, HttpMonitorErrorKind::None);
    } else {
        panic!("Event payload should be MonitorPing");
    }

    let switch_to_recovering_event = events.last().expect("Switch to recovering event should exist");
    assert_eq!(switch_to_recovering_event.event_type, IncidentEventType::MonitorSwitchedToRecovering);

    Ok(())
}

#[tokio::test]
async fn test_handle_ping_response_up_with_ongoing_incident() -> anyhow::Result<()> {
    let http_monitor_repo = HttpMonitorRepositoryMock::new();
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let monitor = create_test_monitor(org_id, HttpMonitorStatus::Up);

    let mut tx = http_monitor_repo.begin_transaction().await?;

    // Add monitor and ongoing incident to repositories
    {
        let mut monitor_state = http_monitor_repo.state.lock().await;
        monitor_state.push(monitor.clone());

        let mut incident_state = incident_repo.state.lock().await;
        incident_state.push(create_test_incident(
            org_id,
            &monitor,
            IncidentStatus::Ongoing,
        ));
    }

    let use_case = ExecuteHttpMonitorsUseCase {
        http_monitor_repository: http_monitor_repo,
        incident_repository: incident_repo,
        incident_event_repository: incident_event_repo,
        incident_notification_repository: incident_notification_repo,
        http_client: HttpClientMock::new(),
        file_storage: FileStorageMock,
    };

    let ping_response = PingResponse {
        error_kind: HttpMonitorErrorKind::None,
        http_code: Some(200),
        http_headers: Default::default(),
        response_time: std::time::Duration::from_secs(1),
        response_ip_address: None,
        resolved_ip_addresses: vec![],
        response_body_size_bytes: 0,
        response_body_content: None,
        screenshot: None,
    };

    let existing_incident = use_case
        .incident_repository
        .state
        .lock()
        .await
        .first()
        .cloned();
    use_case
        .handle_ping_response(&mut tx, monitor, ping_response, existing_incident)
        .await?;

    // Verify incident was resolved
    let incident_state = use_case.incident_repository.state.lock().await;
    let updated_incident = incident_state.first().expect("Incident should exist");
    assert_eq!(updated_incident.status, IncidentStatus::Resolved);
    assert!(updated_incident.resolved_at.is_some());

    // Verify a ping event was created
    let events = use_case.incident_event_repository.state.lock().await;
    assert_eq!(events.len(), 2); // One ping event and one resolution event
    let ping_event = events
        .iter()
        .find(|e| e.event_type == IncidentEventType::MonitorPinged)
        .expect("Ping event should exist");

    if let Some(IncidentEventPayload::MonitorPing(payload)) = &ping_event.event_payload {
        assert_eq!(payload.http_code, Some(200));
        assert_eq!(payload.error_kind, HttpMonitorErrorKind::None);
    } else {
        panic!("Event payload should be MonitorPing");
    }

    Ok(())
}

#[tokio::test]
async fn test_handle_ping_response_recovering_without_status_change() -> anyhow::Result<()> {
    let http_monitor_repo = HttpMonitorRepositoryMock::new();
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let mut monitor = create_test_monitor(org_id, HttpMonitorStatus::Recovering);
    monitor.status_counter = 1; // Already in recovering state for a while

    let mut tx = http_monitor_repo.begin_transaction().await?;

    // Add monitor and incident to repositories
    {
        let mut monitor_state = http_monitor_repo.state.lock().await;
        monitor_state.push(monitor.clone());

        let mut incident_state = incident_repo.state.lock().await;
        incident_state.push(create_test_incident(
            org_id,
            &monitor,
            IncidentStatus::Ongoing,
        ));
    }

    let use_case = ExecuteHttpMonitorsUseCase {
        http_monitor_repository: http_monitor_repo,
        incident_repository: incident_repo,
        incident_event_repository: incident_event_repo,
        incident_notification_repository: incident_notification_repo,
        http_client: HttpClientMock::new(),
        file_storage: FileStorageMock,
    };

    let ping_response = PingResponse {
        error_kind: HttpMonitorErrorKind::None,
        http_code: Some(200),
        http_headers: Default::default(),
        response_time: std::time::Duration::from_secs(1),
        response_ip_address: None,
        resolved_ip_addresses: vec![],
        response_body_size_bytes: 0,
        response_body_content: None,
        screenshot: None,
    };

    let existing_incident = use_case
        .incident_repository
        .state
        .lock()
        .await
        .first()
        .cloned();
    use_case
        .handle_ping_response(&mut tx, monitor, ping_response, existing_incident)
        .await?;

    // Verify monitor remains in recovering state with incremented counter
    let monitor_state = use_case.http_monitor_repository.state.lock().await;
    let updated_monitor = monitor_state.first().expect("Monitor should exist");
    assert_eq!(updated_monitor.status, HttpMonitorStatus::Recovering);
    assert_eq!(updated_monitor.status_counter, 2);

    // Verify no new events were created since this wasn't the first recovering ping
    let events = use_case.incident_event_repository.state.lock().await;
    assert_eq!(events.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_handle_ping_response_suspicious_with_ongoing_incident() -> anyhow::Result<()> {
    let http_monitor_repo = HttpMonitorRepositoryMock::new();
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let monitor = create_test_monitor(org_id, HttpMonitorStatus::Up);

    let mut tx = http_monitor_repo.begin_transaction().await?;

    // Add monitor and ongoing incident to repositories
    {
        let mut monitor_state = http_monitor_repo.state.lock().await;
        monitor_state.push(monitor.clone());

        let mut incident_state = incident_repo.state.lock().await;
        let mut incident = create_test_incident(org_id, &monitor, IncidentStatus::Ongoing);
        incident.cause = Some(IncidentCause::HttpMonitorIncidentCause(
            HttpMonitorIncidentCause {
                last_ping: HttpMonitorIncidentCausePing {
                    error_kind: HttpMonitorErrorKind::HttpCode,
                    http_code: Some(500),
                },
                previous_pings: vec![],
            },
        ));
        incident_state.push(incident);
    }

    let use_case = ExecuteHttpMonitorsUseCase {
        http_monitor_repository: http_monitor_repo,
        incident_repository: incident_repo,
        incident_event_repository: incident_event_repo,
        incident_notification_repository: incident_notification_repo,
        http_client: HttpClientMock::new(),
        file_storage: FileStorageMock,
    };

    let ping_response = PingResponse {
        error_kind: HttpMonitorErrorKind::Timeout,
        http_code: None,
        http_headers: Default::default(),
        response_time: std::time::Duration::from_secs(1),
        response_ip_address: None,
        resolved_ip_addresses: vec![],
        response_body_size_bytes: 0,
        response_body_content: None,
        screenshot: None,
    };

    let existing_incident = use_case
        .incident_repository
        .state
        .lock()
        .await
        .first()
        .cloned();
    use_case
        .handle_ping_response(&mut tx, monitor, ping_response, existing_incident)
        .await?;

    // Verify monitor status changed to Suspicious
    let monitor_state = use_case.http_monitor_repository.state.lock().await;
    let updated_monitor = monitor_state.first().expect("Monitor should exist");
    assert_eq!(updated_monitor.status, HttpMonitorStatus::Suspicious);
    assert_eq!(updated_monitor.error_kind, HttpMonitorErrorKind::Timeout);

    // Verify incident cause was updated
    let incident_state = use_case.incident_repository.state.lock().await;
    let updated_incident = incident_state.first().expect("Incident should exist");
    if let Some(IncidentCause::HttpMonitorIncidentCause(cause)) = &updated_incident.cause {
        assert_eq!(cause.last_ping.error_kind, HttpMonitorErrorKind::Timeout);
        assert_eq!(cause.last_ping.http_code, None);
        assert_eq!(cause.previous_pings.len(), 1);
        assert_eq!(
            cause.previous_pings[0],
            HttpMonitorIncidentCausePing {
                error_kind: HttpMonitorErrorKind::HttpCode,
                http_code: Some(500)
            }
        );

        // verify that a ping event and a switch to suspicious event were created
        let events = use_case.incident_event_repository.state.lock().await;
        assert_eq!(events.len(), 2, "Two events should be created");
        let ping_event = events.first().expect("Ping event should exist");
        assert_eq!(ping_event.event_type, IncidentEventType::MonitorPinged);

        if let Some(IncidentEventPayload::MonitorPing(payload)) = &ping_event.event_payload {
            assert_eq!(payload.error_kind, HttpMonitorErrorKind::Timeout);
            assert_eq!(payload.http_code, None);
        } else {
            panic!("Event payload should be MonitorPing");
        }

        let switch_to_suspicious_event = events.last().expect("Switch to suspicious event should exist");
        assert_eq!(switch_to_suspicious_event.event_type, IncidentEventType::MonitorSwitchedToSuspicious);
    } else {
        panic!("Incident cause should be HttpMonitorIncidentCause");
    }

    Ok(())
}

#[tokio::test]
async fn test_handle_ping_response_unknown_to_suspicious_to_up() -> anyhow::Result<()> {
    let http_monitor_repo = HttpMonitorRepositoryMock::new();
    let incident_repo = IncidentRepositoryMock::new();
    let incident_event_repo = IncidentEventRepositoryMock::new();
    let incident_notification_repo = IncidentNotificationRepositoryMock::new();

    let org_id = Uuid::new_v4();
    let mut monitor = create_test_monitor(org_id, HttpMonitorStatus::Unknown);
    monitor.recovery_confirmation_threshold = 2;
    monitor.downtime_confirmation_threshold = 2;

    let mut tx = http_monitor_repo.begin_transaction().await?;

    // Add monitor to repository
    {
        let mut monitor_state = http_monitor_repo.state.lock().await;
        monitor_state.push(monitor.clone());
    }

    let use_case = ExecuteHttpMonitorsUseCase {
        http_monitor_repository: http_monitor_repo,
        incident_repository: incident_repo,
        incident_event_repository: incident_event_repo,
        incident_notification_repository: incident_notification_repo,
        http_client: HttpClientMock::new(),
        file_storage: FileStorageMock,
    };

    // First ping - HTTP 500
    let ping_response = PingResponse {
        error_kind: HttpMonitorErrorKind::HttpCode,
        http_code: Some(500),
        http_headers: Default::default(),
        response_time: std::time::Duration::from_secs(1),
        response_ip_address: None,
        resolved_ip_addresses: vec![],
        response_body_size_bytes: 0,
        response_body_content: None,
        screenshot: None,
    };

    use_case
        .handle_ping_response(&mut tx, monitor.clone(), ping_response, None)
        .await?;

    // Verify monitor status changed to Suspicious
    {
        let monitor_state = use_case.http_monitor_repository.state.lock().await;
        let updated_monitor = monitor_state.first().expect("Monitor should exist");
        assert_eq!(updated_monitor.status, HttpMonitorStatus::Suspicious);
        assert_eq!(updated_monitor.status_counter, 1);
        assert_eq!(updated_monitor.error_kind, HttpMonitorErrorKind::HttpCode);
        assert_eq!(updated_monitor.last_http_code, Some(500));
    }

    // Verify an unconfirmed incident was created
    {
        let incident_state = use_case.incident_repository.state.lock().await;
        let incident = incident_state.first().expect("Incident should exist");
        assert_eq!(incident.status, IncidentStatus::ToBeConfirmed);

        // Verify incident cause
        if let Some(IncidentCause::HttpMonitorIncidentCause(cause)) = &incident.cause {
            assert_eq!(cause.last_ping.error_kind, HttpMonitorErrorKind::HttpCode);
            assert_eq!(cause.last_ping.http_code, Some(500));
            assert!(cause.previous_pings.is_empty());
        } else {
            panic!("Incident cause should be HttpMonitorIncidentCause");
        }
    }

    // Second ping - HTTP 200
    let ping_response = PingResponse {
        error_kind: HttpMonitorErrorKind::None,
        http_code: Some(200),
        http_headers: Default::default(),
        response_time: std::time::Duration::from_secs(1),
        response_ip_address: None,
        resolved_ip_addresses: vec![],
        response_body_size_bytes: 0,
        response_body_content: None,
        screenshot: None,
    };

    let existing_incident = use_case
        .incident_repository
        .state
        .lock()
        .await
        .first()
        .cloned();

    use_case
        .handle_ping_response(&mut tx, monitor, ping_response, existing_incident)
        .await?;

    // Verify monitor status changed to Up
    {
        let monitor_state = use_case.http_monitor_repository.state.lock().await;
        let updated_monitor = monitor_state.first().expect("Monitor should exist");
        assert_eq!(updated_monitor.status, HttpMonitorStatus::Up);
        assert_eq!(updated_monitor.status_counter, 1);
        assert_eq!(updated_monitor.error_kind, HttpMonitorErrorKind::None);
        assert_eq!(updated_monitor.last_http_code, Some(200));
    }

    // Verify incident was deleted
    {
        let incident_state = use_case.incident_repository.state.lock().await;
        assert_eq!(incident_state.len(), 0);
    }

    Ok(())
}
