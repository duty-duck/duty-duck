use crate::{domain::{
    entities::{authorization::AuthContext, organization::OrganizationUserRole},
    use_cases::tasks::create_task_use_case::{create_task, CreateTaskCommand, CreateTaskError},
}, infrastructure::mocks::task_repository_mock::TaskRepositoryMock};
use uuid::Uuid;

fn create_basic_command() -> CreateTaskCommand {
    CreateTaskCommand {
        id: "test-task".to_string(),
        name: "Test Task".to_string(),
        description: None,
        is_active: true,
        cron_schedule: None,
        start_window_seconds: 300,
        lateness_window_seconds: 600,
        heartbeat_timeout_seconds: 60,
    }
}

#[tokio::test]
async fn test_create_task_without_permission() {
    let auth_context = AuthContext::test_context(Uuid::new_v4(), Uuid::new_v4(), &[], &[]);
    let repository = TaskRepositoryMock::new();
    let command = create_basic_command();

    let result = create_task(&auth_context, &repository, command).await;
    assert!(matches!(result, Err(CreateTaskError::Forbidden)));
}

#[tokio::test]
async fn test_create_task_with_invalid_id() {
    let auth_context = AuthContext::test_context(Uuid::new_v4(), Uuid::new_v4(), &[OrganizationUserRole::Editor], &[]);
    let repository = TaskRepositoryMock::new();
    let mut command = create_basic_command();
    command.id = "invalid task id with spaces".to_string();

    let result = create_task(&auth_context, &repository, command).await;
    assert!(matches!(result, Err(CreateTaskError::InvalidTaskId)));
}

#[tokio::test]
async fn test_create_task_with_invalid_cron() {
    let auth_context = AuthContext::test_context(Uuid::new_v4(), Uuid::new_v4(), &[OrganizationUserRole::Editor], &[]);
    let repository = TaskRepositoryMock::new();
    let mut command = create_basic_command();
    command.cron_schedule = Some("invalid cron".to_string());

    let result = create_task(&auth_context, &repository, command).await;
    assert!(matches!(
        result,
        Err(CreateTaskError::InvalidCronExpression(_))
    ));
}

#[tokio::test]
async fn test_create_task_success() {
    let auth_context = AuthContext::test_context(Uuid::new_v4(), Uuid::new_v4(), &[OrganizationUserRole::Editor], &[]);
    let repository = TaskRepositoryMock::new();
    let mut command = create_basic_command();
    command.description = Some("Description".to_string());
    command.cron_schedule = Some("0 0 * * *".to_string());

    let result = create_task(&auth_context, &repository, command).await;
    assert!(result.is_ok());

    let created_task_id = result.unwrap().id;
    assert_eq!(created_task_id.as_str(), "test-task");
}

#[tokio::test]
async fn test_create_inactive_task() {
    let auth_context = AuthContext::test_context(Uuid::new_v4(), Uuid::new_v4(), &[OrganizationUserRole::Editor], &[]);
    let repository = TaskRepositoryMock::new();
    let mut command = create_basic_command();
    command.is_active = false;

    let result = create_task(&auth_context, &repository, command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_task_with_cron_schedule() {
    let auth_context = AuthContext::test_context(Uuid::new_v4(), Uuid::new_v4(), &[OrganizationUserRole::Editor], &[]);
    let repository = TaskRepositoryMock::new();
    let mut command = create_basic_command();
    command.cron_schedule = Some("*/15 * * * *".to_string());

    let result = create_task(&auth_context, &repository, command).await;
    assert!(result.is_ok());
}
