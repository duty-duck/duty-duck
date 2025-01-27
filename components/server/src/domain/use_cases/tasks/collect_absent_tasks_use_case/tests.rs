use crate::{
    domain::{
        entities::{
            entity_metadata::{EntityMetadata, MetadataFilter},
            incident::{IncidentCause, IncidentPriority, IncidentSource, IncidentStatus, NewIncident, ScheduledTaskIncidentCause},
            incident_event::{IncidentEvent, IncidentEventType},
            task::{BoundaryTask, TaskId, TaskStatus},
        },
        ports::{
            incident_event_repository::IncidentEventRepository,
            incident_repository::{IncidentRepository, ListIncidentsOpts},
            task_repository::TaskRepository,
            transactional_repository::TransactionalRepository,
        },
        use_cases::{incidents::OrderIncidentsBy, shared::OrderDirection},
    },
    infrastructure::mocks::{
        incident_event_repository_mock::IncidentEventRepositoryMock,
        incident_notification_repository_mock::IncidentNotificationRepositoryMock,
        incident_repository_mock::IncidentRepositoryMock, task_repository_mock::TaskRepositoryMock,
        task_run_repository_mock::TaskRunRepositoryMock,
    },
};

use super::CollectAbsentTasksUseCase;
use anyhow::Context;
use chrono::*;
use uuid::Uuid;

fn build_use_case() -> CollectAbsentTasksUseCase<
    TaskRepositoryMock,
    TaskRunRepositoryMock,
    IncidentRepositoryMock,
    IncidentEventRepositoryMock,
    IncidentNotificationRepositoryMock,
> {
    CollectAbsentTasksUseCase {
        task_repository: TaskRepositoryMock::new(),
        task_run_repository: TaskRunRepositoryMock::new(),
        incident_repository: IncidentRepositoryMock::new(),
        incident_event_repository: IncidentEventRepositoryMock::new(),
        incident_notification_repository: IncidentNotificationRepositoryMock::new(),
        select_limit: 10,
    }
}


/// A test for when a task runs from late to absent and has no existing incident
/// This test verifies that the task is updated to Absent and a new incident is created with all the events
/// from a task running late (incident creation, task switched to due, task switched to late) and an additional event for task switched to absent
#[tokio::test]
async fn test_collect_absent_tasks_updates_existing_incident() -> anyhow::Result<()> {
    let use_case = build_use_case();
    let org_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let task_user_id = TaskId::new("test-task".to_string()).context("Failed to create task id")?;

    let task_created_at = Utc.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap(); // task was created at 10:00
    let task_was_due_at = Utc.with_ymd_and_hms(2025, 1, 1, 10, 30, 0).unwrap(); // task was due to run at 10:30
    let task_ran_late_at = Utc.with_ymd_and_hms(2025, 1, 1, 10, 50, 0).unwrap(); // task ran late at 10:50
    let now = Utc.with_ymd_and_hms(2025, 1, 1, 10, 50, 0).unwrap(); // use case is evaluated at 10:50

    // Create a late task that's now absent
    let task = BoundaryTask {
        organization_id: org_id,
        id: task_id,
        user_id: task_user_id.clone(),
        name: "Test Task".to_string(),
        description: None,
        status: TaskStatus::Late,
        previous_status: Some(TaskStatus::Due),
        last_status_change_at: Some(task_ran_late_at),
        cron_schedule: Some("*/30 * * * *".to_string()),
        next_due_at: Some(task_was_due_at),
        start_window_seconds: 300,     // 5 minutes
        lateness_window_seconds: 600,  // 10 minutes
        heartbeat_timeout_seconds: 60,
        created_at: task_created_at,
        metadata: EntityMetadata::default(),
        schedule_timezone: None,
    };

    // Create existing incident for the late task
    let incident_cause = IncidentCause::ScheduledTaskIncidentCause(ScheduledTaskIncidentCause {
        task_id,
        task_user_id: task_user_id.clone(),
        task_was_due_at,
        task_ran_late_at: Some(task_ran_late_at),
        task_switched_to_absent_at: None,
    });

    let new_incident = NewIncident {
        organization_id: org_id,
        created_by: None,
        status: IncidentStatus::Ongoing,
        priority: IncidentPriority::Major,
        source: IncidentSource::Task { id: task_id },
        cause: Some(incident_cause),
        metadata: EntityMetadata::default(),
    };

    // Setup initial state
    let mut tx = use_case.task_repository.begin_transaction().await?;
    use_case.task_repository.upsert_task(&mut tx, task).await?;
    let incident_id = use_case
        .incident_repository
        .create_incident(&mut tx, new_incident)
        .await?;

    // Add initial events
    let creation_event = IncidentEvent {
        organization_id: org_id,
        incident_id,
        user_id: None,
        created_at: task_ran_late_at,
        event_type: IncidentEventType::Creation,
        event_payload: None,
    };
    let due_event = IncidentEvent {
        organization_id: org_id,
        incident_id,
        user_id: None,
        created_at: task_was_due_at,
        event_type: IncidentEventType::TaskSwitchedToDue,
        event_payload: None,
    };
    let late_event = IncidentEvent {
        organization_id: org_id,
        incident_id,
        user_id: None,
        created_at: task_ran_late_at,
        event_type: IncidentEventType::TaskSwitchedToLate,
        event_payload: None,
    };

    use_case
        .incident_event_repository
        .create_incident_event(&mut tx, creation_event)
        .await?;
    use_case
        .incident_event_repository
        .create_incident_event(&mut tx, due_event)
        .await?;
    use_case
        .incident_event_repository
        .create_incident_event(&mut tx, late_event)
        .await?;

    // Run the use case
    let absent_tasks = use_case.collect_absent_tasks(now).await?;
    assert_eq!(absent_tasks, 1);

    // Verify task was updated to Absent
    let mut tx = use_case.task_repository.begin_transaction().await?;
    let updated_task = use_case
        .task_repository
        .get_task_by_user_id(&mut tx, org_id, &task_user_id)
        .await?
        .expect("Task should exist");
    assert_eq!(updated_task.status, TaskStatus::Absent);

    // Verify incident was updated with new event
    let events = use_case
        .incident_event_repository
        .get_incident_timeline(org_id, incident_id, 10, 0)
        .await?;

    assert_eq!(events.len(), 4); // Original 3 events + new absent event
    assert_eq!(
        events.last().expect("Should have events").event_type,
        IncidentEventType::TaskSwitchedToAbsent
    );

    Ok(())
}

#[tokio::test]
async fn test_collect_absent_tasks_creates_new_incident() -> anyhow::Result<()> {
    let use_case = build_use_case();
    let org_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let task_user_id = TaskId::new("test-task".to_string()).context("Failed to create task id")?;

    let task_created_at = Utc.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap(); // task was created at 10:00
    let task_was_due_at = Utc.with_ymd_and_hms(2025, 1, 1, 10, 30, 0).unwrap(); // task was due to run at 10:30
    let now = Utc.with_ymd_and_hms(2025, 1, 1, 10, 50, 0).unwrap(); // use case is evaluated at 10:50

    // Create a late task that's now absent (but with no existing incident)
    let task = BoundaryTask {
        organization_id: org_id,
        id: task_id,
        user_id: task_user_id.clone(),
        name: "Test Task".to_string(),
        description: None,
        status: TaskStatus::Late,
        previous_status: Some(TaskStatus::Due),
        last_status_change_at: Some(task_was_due_at),
        cron_schedule: Some("*/30 * * * *".to_string()),
        next_due_at: Some(task_was_due_at),
        start_window_seconds: 300,     // 5 minutes
        lateness_window_seconds: 600,  // 10 minutes
        heartbeat_timeout_seconds: 60,
        created_at: task_created_at,
        metadata: EntityMetadata::default(),
        schedule_timezone: None,
    };

    // Setup initial state
    let mut tx = use_case.task_repository.begin_transaction().await?;
    use_case.task_repository.upsert_task(&mut tx, task).await?;
    use_case.task_repository.commit_transaction(tx).await?;

    // Run the use case
    let absent_tasks = use_case.collect_absent_tasks(now).await?;
    assert_eq!(absent_tasks, 1);

    // Verify task was updated to Absent
    let mut tx = use_case.task_repository.begin_transaction().await?;
    let updated_task = use_case
        .task_repository
        .get_task_by_user_id(&mut tx, org_id, &task_user_id)
        .await?
        .expect("Task should exist");
    assert_eq!(updated_task.status, TaskStatus::Absent);

    // Verify new incident was created
    let incidents = use_case
        .incident_repository
        .list_incidents(
            &mut tx,
            org_id,
            ListIncidentsOpts {
                include_statuses: &[],
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

    assert_eq!(incidents.incidents.len(), 1);
    let incident = &incidents.incidents[0];

    // Verify incident cause
    if let Some(IncidentCause::ScheduledTaskIncidentCause(cause)) = &incident.cause {
        assert_eq!(cause.task_id, task_id);
        assert_eq!(cause.task_user_id, task_user_id);
        assert!(cause.task_switched_to_absent_at.is_some());
    } else {
        panic!("Expected ScheduledTaskIncidentCause");
    }

    // Verify all events were created
    let events = use_case
        .incident_event_repository
        .get_incident_timeline(org_id, incident.id, 10, 0)
        .await?;

    assert_eq!(events.len(), 4);

    // Events should be in chronological order
    let event_types: Vec<_> = events.iter().map(|e| e.event_type).collect();
    assert_eq!(
        event_types,
        vec![
            IncidentEventType::TaskSwitchedToDue,
            IncidentEventType::Creation,
            IncidentEventType::TaskSwitchedToLate,
            IncidentEventType::TaskSwitchedToAbsent,
        ]
    );

    Ok(())
}