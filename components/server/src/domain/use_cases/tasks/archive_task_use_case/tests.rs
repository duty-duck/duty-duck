// src/domain/use_cases/tasks/archive_task_use_case/tests.rs
use anyhow::Context;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    domain::{
        entities::{
            authorization::AuthContext,
            entity_metadata::EntityMetadata,
            organization::OrganizationUserRole,
            task::{BoundaryTask, TaskId, TaskStatus, TaskUserId},
            task_run::{BoundaryTaskRun, TaskRunStatus},
        },
        ports::{
            task_repository::TaskRepository, task_run_repository::TaskRunRepository,
            transactional_repository::TransactionalRepository,
        },
    },
    infrastructure::mocks::{
        task_repository_mock::TaskRepositoryMock, task_run_repository_mock::TaskRunRepositoryMock,
    },
};

#[tokio::test]
async fn test_archive_task_successful() -> anyhow::Result<()> {
    let organization_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let task_user_id = TaskUserId::new("test-task").unwrap();

    let task_repository = TaskRepositoryMock::new();
    let task_run_repository = TaskRunRepositoryMock::new();
    let now = Utc::now();

    let task = BoundaryTask {
        organization_id,
        id: task_id,
        user_id: task_user_id.clone(),
        name: "Test Task".to_string(),
        description: None,
        status: TaskStatus::Healthy,
        previous_status: None,
        last_status_change_at: Some(now),
        cron_schedule: Some("*/30 * * * *".to_string()),
        next_due_at: Some(now),
        start_window_seconds: 300,
        lateness_window_seconds: 600,
        heartbeat_timeout_seconds: 60,
        created_at: now - chrono::Duration::hours(1),
        metadata: EntityMetadata::default(),
        schedule_timezone: None,
        email_notification_enabled: true,
        push_notification_enabled: true,
        sms_notification_enabled: false,
    };

    let mut tx = task_repository.begin_transaction().await?;
    task_repository.upsert_task(&mut tx, task).await?;

    let auth_context = AuthContext::test_context(
        organization_id,
        user_id,
        &[OrganizationUserRole::Owner],
        &[],
    );

    super::archive_task(
        &task_repository,
        &task_run_repository,
        &auth_context,
        TaskId::UserId(task_user_id.clone()),
    )
    .await?;

    // The task should be archived
    // User ids cannot be used to retrieve archived tasks, so this should return None.
    assert!(task_repository
        .get_task_by_user_id(&mut tx, organization_id, &task_user_id)
        .await?
        .is_none());

    let archived_task = task_repository
        .get_task_by_uuid(&mut tx, organization_id, task_id)
        .await?
        .context("Task not found")?;
    assert_eq!(archived_task.status, TaskStatus::Archived);

    Ok(())
}

/// When trying to archive an archived task using its user id, the error should be "task not found".
/// Indeed, the user id cannot be used to retrieve an archived task.
#[tokio::test]
async fn test_archive_task_already_archived_by_its_user_id() -> anyhow::Result<()> {
    let organization_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let task_user_id = TaskUserId::new("test-task").unwrap();

    let task_repository = TaskRepositoryMock::new();
    let task_run_repository = TaskRunRepositoryMock::new();
    let now = Utc::now();

    let task = BoundaryTask {
        organization_id,
        id: task_id,
        user_id: task_user_id.clone(),
        name: "Test Task".to_string(),
        description: None,
        status: TaskStatus::Archived,
        previous_status: None,
        last_status_change_at: Some(now),
        cron_schedule: Some("*/30 * * * *".to_string()),
        next_due_at: Some(now),
        start_window_seconds: 300,
        lateness_window_seconds: 600,
        heartbeat_timeout_seconds: 60,
        created_at: now - chrono::Duration::hours(1),
        metadata: EntityMetadata::default(),
        schedule_timezone: None,
        email_notification_enabled: true,
        push_notification_enabled: true,
        sms_notification_enabled: false,
    };

    let mut tx = task_repository.begin_transaction().await?;
    task_repository.upsert_task(&mut tx, task).await?;
    task_repository.commit_transaction(tx).await?;

    let auth_context = AuthContext::test_context(
        organization_id,
        user_id,
        &[OrganizationUserRole::Owner],
        &[],
    );

    let result = super::archive_task(
        &task_repository,
        &task_run_repository,
        &auth_context,
        TaskId::UserId(task_user_id),
    )
    .await;
    match result {
        Err(super::ArchiveTaskError::TaskNotFound { .. }) => (),
        other => panic!(
            "Expected ArchiveTaskError::TaskNotFound but got {:?}",
            other
        ),
    };

    Ok(())
}

#[tokio::test]
async fn test_archive_task_already_archived_by_its_uuid() -> anyhow::Result<()> {
    let organization_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let task_user_id = TaskUserId::new("test-task").unwrap();

    let task_repository = TaskRepositoryMock::new();
    let task_run_repository = TaskRunRepositoryMock::new();
    let now = Utc::now();

    let task = BoundaryTask {
        organization_id,
        id: task_id,
        user_id: task_user_id.clone(),
        name: "Test Task".to_string(),
        description: None,
        status: TaskStatus::Archived,
        previous_status: None,
        last_status_change_at: Some(now),
        cron_schedule: Some("*/30 * * * *".to_string()),
        next_due_at: Some(now),
        start_window_seconds: 300,
        lateness_window_seconds: 600,
        heartbeat_timeout_seconds: 60,
        created_at: now - chrono::Duration::hours(1),
        metadata: EntityMetadata::default(),
        schedule_timezone: None,
        email_notification_enabled: true,
        push_notification_enabled: true,
        sms_notification_enabled: false,
    };

    let mut tx = task_repository.begin_transaction().await?;
    task_repository.upsert_task(&mut tx, task).await?;
    task_repository.commit_transaction(tx).await?;

    let auth_context = AuthContext::test_context(
        organization_id,
        user_id,
        &[OrganizationUserRole::Owner],
        &[],
    );

    let result = super::archive_task(
        &task_repository,
        &task_run_repository,
        &auth_context,
        TaskId::Uuid(task_id),
    )
    .await;
    match result {
        Err(super::ArchiveTaskError::AlreadyArchived { .. }) => (),
        other => panic!(
            "Expected ArchiveTaskError::AlreadyArchived but got {:?}",
            other
        ),
    };

    Ok(())
}

#[tokio::test]
async fn test_archive_task_running() -> anyhow::Result<()> {
    let organization_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let task_user_id = TaskUserId::new("test-task").unwrap();

    let task_repository = TaskRepositoryMock::new();
    let task_run_repository = TaskRunRepositoryMock::new();
    let now = Utc::now();

    let task = BoundaryTask {
        organization_id,
        id: task_id,
        user_id: task_user_id.clone(),
        name: "Test Task".to_string(),
        description: None,
        status: TaskStatus::Running,
        previous_status: None,
        last_status_change_at: Some(now),
        cron_schedule: Some("*/30 * * * *".to_string()),
        next_due_at: Some(now),
        start_window_seconds: 300,
        lateness_window_seconds: 600,
        heartbeat_timeout_seconds: 60,
        created_at: now - chrono::Duration::hours(1),
        metadata: EntityMetadata::default(),
        schedule_timezone: None,
        email_notification_enabled: true,
        push_notification_enabled: true,
        sms_notification_enabled: false,
    };

    let mut tx = task_repository.begin_transaction().await?;
    task_repository.upsert_task(&mut tx, task).await?;

    task_run_repository
        .upsert_task_run(
            &mut tx,
            BoundaryTaskRun {
                organization_id,
                id: Uuid::new_v4(),
                task_id,
                status: TaskRunStatus::Running,
                started_at: now,
                updated_at: now,
                completed_at: None,
                exit_code: None,
                error_message: None,
                last_heartbeat_at: Some(now),
                heartbeat_timeout_seconds: 60,
                metadata: EntityMetadata::default(),
            },
        )
        .await?;

    let auth_context = AuthContext::test_context(
        organization_id,
        user_id,
        &[OrganizationUserRole::Owner],
        &[],
    );

    let result = super::archive_task(
        &task_repository,
        &task_run_repository,
        &auth_context,
        TaskId::UserId(task_user_id),
    )
    .await;
    match result {
        Err(super::ArchiveTaskError::CannotArchiveRunningTask) => (),
        other => panic!(
            "Expected CannotArchiveRunningTask error but got {:?}",
            other
        ),
    }

    Ok(())
}
