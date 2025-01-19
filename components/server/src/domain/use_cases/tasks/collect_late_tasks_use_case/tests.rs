use crate::{
    domain::{
        entities::{
            entity_metadata::{EntityMetadata, MetadataFilter},
            incident::{IncidentCause, IncidentSourceType},
            incident_event::IncidentEventType,
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

use super::CollectLateTasksUseCase;
use anyhow::Context;
use chrono::*;
use uuid::Uuid;

fn build_use_case() -> CollectLateTasksUseCase<
    TaskRepositoryMock,
    TaskRunRepositoryMock,
    IncidentRepositoryMock,
    IncidentEventRepositoryMock,
    IncidentNotificationRepositoryMock,
> {
    CollectLateTasksUseCase {
        task_repository: TaskRepositoryMock::new(),
        task_run_repository: TaskRunRepositoryMock::new(),
        incident_repository: IncidentRepositoryMock::new(),
        incident_event_repository: IncidentEventRepositoryMock::new(),
        incident_notification_repository: IncidentNotificationRepositoryMock::new(),
        select_limit: 10,
    }
}

#[tokio::test]
async fn test_collect_late_tasks_creates_incident_with_events() -> anyhow::Result<()> {
    let use_case = build_use_case();
    let org_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let task_user_id = TaskId::new("test-task".to_string()).context("Failed to create task id")?;

    let task_created_at = Utc.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap(); // task was created at 10:00
    let task_was_due_at = Utc.with_ymd_and_hms(2025, 1, 1, 10, 30, 0).unwrap(); // task was due to run at 10:30
    let now = Utc.with_ymd_and_hms(2025, 1, 1, 10, 50, 0).unwrap(); // use case is evaluated at 10:50

    // Create a due task that's running late
    let task = BoundaryTask {
        organization_id: org_id,
        id: task_id,
        user_id: task_user_id.clone(),
        name: "Test Task".to_string(),
        description: None,
        status: TaskStatus::Due,
        previous_status: None,
        last_status_change_at: Some(task_created_at),
        cron_schedule: Some("*/30 * * * *".to_string()),
        next_due_at: Some(task_was_due_at),
        start_window_seconds: 300,    // 5 minutes
        lateness_window_seconds: 600, // 10 minutes
        heartbeat_timeout_seconds: 60,
        created_at: now - Duration::hours(1),
        metadata: EntityMetadata::default(),
        schedule_timezone: None,
    };

    // Add the task to the repository
    let mut tx = use_case.task_repository.begin_transaction().await?;
    use_case.task_repository.upsert_task(&mut tx, task).await?;
    use_case.task_repository.commit_transaction(tx).await?;

    // Run the use case
    let late_tasks = use_case.collect_late_tasks(now).await?;
    assert_eq!(late_tasks, 1);

    // Verify the task was updated to Late status
    let mut tx = use_case.task_repository.begin_transaction().await?;
    let updated_task = use_case
        .task_repository
        .get_task_by_user_id(&mut tx, org_id, &task_user_id)
        .await
        .context("Failed to get task")?
        .context("Task should exist")?;

    assert_eq!(updated_task.status, TaskStatus::Late);

    // Get the created incident
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

    // Verify incident details
    assert_eq!(incident.organization_id, org_id);
    assert_eq!(incident.incident_source_type, IncidentSourceType::Task);
    assert_eq!(incident.incident_source_id, task_id);

    // Verify incident cause
    if let Some(IncidentCause::ScheduledTaskIncidentCause(cause)) = &incident.cause {
        assert_eq!(cause.task_id, task_id);
        assert_eq!(cause.task_user_id, task_user_id);
        assert_eq!(cause.task_was_due_at, task_was_due_at);
        assert_eq!(cause.task_ran_late_at, Some(now));
        assert_eq!(cause.task_switched_to_absent_at, None);
    } else {
        panic!("Expected ScheduledTaskIncidentCause");
    }

    // Verify incident events
    let events = use_case
        .incident_event_repository
        .get_incident_timeline(org_id, incident.id, 10, 0)
        .await?;

    // There should be 3 events: creation, task switched to due, and task switched to late
    assert_eq!(events.len(), 3);

    // the first event should be the "task switched to due" event
    assert_eq!(events[0].event_type, IncidentEventType::TaskSwitchedToDue);
    assert_eq!(events[0].created_at, task_was_due_at);

    // the second event should be the event creation
    assert_eq!(events[1].event_type, IncidentEventType::Creation);
    assert_eq!(events[1].created_at, now);

    // the third event should be the "task switched to late" event
    assert_eq!(events[2].event_type, IncidentEventType::TaskSwitchedToLate);
    assert_eq!(events[2].created_at, now);

    Ok(())
}
