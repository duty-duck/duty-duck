use anyhow::Context;
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    domain::{
        entities::{
            authorization::AuthContext, entity_metadata::EntityMetadata, incident::{
                IncidentCause, IncidentPriority, IncidentSource, IncidentStatus, NewIncident,
                ScheduledTaskIncidentCause,
            }, incident_event::{IncidentEvent, IncidentEventType}, organization::OrganizationUserRole, task::{BoundaryTask, TaskId, TaskStatus}, task_run::TaskRunStatus
        },
        ports::{
            incident_event_repository::IncidentEventRepository, incident_repository::IncidentRepository, task_repository::TaskRepository, task_run_repository::TaskRunRepository, transactional_repository::TransactionalRepository
        },
    },
    infrastructure::mocks::{
        incident_event_repository_mock::IncidentEventRepositoryMock,
        incident_notification_repository_mock::IncidentNotificationRepositoryMock,
        incident_repository_mock::IncidentRepositoryMock, task_repository_mock::TaskRepositoryMock,
        task_run_repository_mock::TaskRunRepositoryMock,
    },
};

#[tokio::test]
async fn test_start_task_use_case_with_late_task() -> anyhow::Result<()> {
    let organization_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let task_user_id = TaskId::new("test-task".to_string()).context("Failed to create task id")?;

    let now = Utc::now();
    let task_was_due_at = now - Duration::minutes(10);
    let task_ran_late_at = now - Duration::minutes(5);

    let task_repository = TaskRepositoryMock::new();
    let task_run_repository = TaskRunRepositoryMock::new();
    let incident_repository = IncidentRepositoryMock::new();
    let incident_event_repository = IncidentEventRepositoryMock::new();
    let incident_notification_repository = IncidentNotificationRepositoryMock::new();
    let mut tx = task_repository.begin_transaction().await?;

    // Setup: Create a task that is late
    let task = BoundaryTask {
        organization_id,
        id: task_id,
        user_id: task_user_id.clone(),
        name: "Test Task".to_string(),
        description: None,
        status: TaskStatus::Late,
        previous_status: None,
        last_status_change_at: Some(task_ran_late_at),
        cron_schedule: Some("*/30 * * * *".to_string()),
        next_due_at: Some(task_was_due_at),
        start_window_seconds: 300,    // 5 minutes
        lateness_window_seconds: 600, // 10 minutes
        heartbeat_timeout_seconds: 60,
        created_at: now - Duration::hours(1),
        metadata: EntityMetadata::default(),
        schedule_timezone: None,
        email_notification_enabled: true,
        push_notification_enabled: true,
        sms_notification_enabled: false,
    };

    // Setup: Add the task to the repository
    task_repository.upsert_task(&mut tx, task).await?;

    // Setup: Create existing incident for the late task
    let incident_cause = IncidentCause::ScheduledTaskIncidentCause(ScheduledTaskIncidentCause {
        task_id,
        task_user_id: task_user_id.clone(),
        task_was_due_at,
        task_ran_late_at: Some(task_ran_late_at),
        task_switched_to_absent_at: None,
    });

    let new_incident = NewIncident {
        organization_id,
        created_by: None,
        status: IncidentStatus::Ongoing,
        priority: IncidentPriority::Major,
        source: IncidentSource::Task { id: task_id },
        cause: Some(incident_cause),
        metadata: EntityMetadata::default(),
    };

    // Setup initial state
    let incident_id = incident_repository
        .create_incident(&mut tx, new_incident)
        .await?;

    // Setup: Add 1 initial event to the incident
    let creation_event = IncidentEvent {
        organization_id,
        incident_id,
        user_id: None,
        created_at: task_ran_late_at,
        event_type: IncidentEventType::Creation,
        event_payload: None,
    };

    incident_event_repository
        .create_incident_event(&mut tx, creation_event)
        .await?;

    // Run the use case
    let auth_context = AuthContext::test_context(organization_id, user_id, &[OrganizationUserRole::Owner], &[]);
    super::start_task_use_case(&auth_context, super::StartTaskUseCaseOpts {
        task_repository: &task_repository,
        task_run_repository: &task_run_repository,
        incident_repository: &incident_repository,
        incident_event_repository: &incident_event_repository,
        incident_notification_repository: &incident_notification_repository,
        task_id: task_user_id.clone(),
        command: None
    }).await?;

    // Verify task is now running
    let task = task_repository
        .get_task_by_user_id(&mut tx, organization_id, &task_user_id)
        .await?
        .context("Task not found")?;

    assert_eq!(task.status, TaskStatus::Running);
    assert_eq!(task.previous_status, Some(TaskStatus::Late));

    // Verify task run is created
    let task_run = task_run_repository
        .get_latest_task_run(&mut tx, organization_id, task_id, &[TaskRunStatus::Running])
        .await?
        .context("Task run not found")?;

    assert_eq!(task_run.status, TaskRunStatus::Running);

    // Verify incident has TaskSwitchedToRunning event
    let events = incident_event_repository
        .get_incident_timeline(organization_id, incident_id, 100, 0)
        .await?;

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].event_type, IncidentEventType::Creation);
    assert_eq!(events[1].event_type, IncidentEventType::TaskSwitchedToRunning);
    assert_eq!(events[2].event_type, IncidentEventType::Resolution);

    // Verify incident is resolved
    let incident = incident_repository
        .get_incident(&mut tx, organization_id, incident_id)
        .await?.context("Incident not found")?;

    assert_eq!(incident.status, IncidentStatus::Resolved);

    Ok(())
}
