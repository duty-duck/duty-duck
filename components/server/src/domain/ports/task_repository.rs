use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::task::{BoundaryTask, TaskId, TaskStatus};
use super::transactional_repository::TransactionalRepository;

#[async_trait]
pub trait TaskRepository: TransactionalRepository + Clone + Send + Sync + 'static {
    /// Get a single task by organization ID and task ID
    async fn get_task(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: TaskId,
    ) -> anyhow::Result<Option<BoundaryTask>>;

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
    async fn create_task(&self, task: BoundaryTask) -> anyhow::Result<TaskId>;

    /// Update an existing task
    async fn update_task(
        &self,
        transaction: &mut Self::Transaction,
        task: BoundaryTask
    ) -> anyhow::Result<bool>;

    /// Update task status and related fields
    async fn update_task_status(
        &self,
        transaction: &mut Self::Transaction,
        command: UpdateTaskStatusCommand,
    ) -> anyhow::Result<()>;
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
    pub tasks: Vec<BoundaryTask>,
    pub total_tasks: u32,
    pub total_filtered_tasks: u32,
} 