use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::task::{Task, TaskId, TaskStatus};
use super::transactional_repository::TransactionalRepository;

#[async_trait]
pub trait TaskRepository: TransactionalRepository + Clone + Send + Sync + 'static {
    /// Get a single task by organization ID and task ID
    async fn get_task(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: TaskId,
    ) -> anyhow::Result<Option<Task>>;

    /// List tasks with pagination and filtering
    async fn list_tasks(
        &self,
        organization_id: Uuid,
        include_statuses: Vec<TaskStatus>,
        query: String,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<ListTasksOutput>;

    /// Create a new task
    async fn create_task(&self, task: NewTask) -> anyhow::Result<TaskId>;

    /// Update an existing task
    async fn update_task(
        &self,
        transaction: &mut Self::Transaction,
        id: TaskId,
        task: NewTask,
    ) -> anyhow::Result<bool>;

    /// Update task status and related fields
    async fn update_task_status(
        &self,
        transaction: &mut Self::Transaction,
        command: UpdateTaskStatusCommand,
    ) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct NewTask {
    pub organization_id: Uuid,
    pub id: TaskId,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub cron_schedule: Option<String>,
    pub next_due_at: Option<DateTime<Utc>>,
    pub start_window_seconds: i32,
    pub lateness_window_seconds: i32,
    pub heartbeat_timeout_seconds: i32,
}

#[derive(Debug)]
pub struct UpdateTaskStatusCommand {
    pub organization_id: Uuid,
    pub task_id: TaskId,
    pub status: TaskStatus,
    pub previous_status: TaskStatus,
    pub last_status_change_at: Option<DateTime<Utc>>,
    pub next_due_at: Option<DateTime<Utc>>,
}

pub struct ListTasksOutput {
    pub tasks: Vec<Task>,
    pub total_tasks: u32,
    pub total_filtered_tasks: u32,
} 