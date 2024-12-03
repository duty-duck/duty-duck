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
        task_id: &TaskId,
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

    /// Create or update an existing task
    async fn upsert_task(
        &self,
        transaction: &mut Self::Transaction,
        task: BoundaryTask
    ) -> anyhow::Result<TaskId>;

    /// List scheduled tasks that should transition to Due
    async fn list_due_tasks(
        &self,
        transaction: &mut Self::Transaction,
        now: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<BoundaryTask>>;
}

pub struct ListTasksOutput {
    pub tasks: Vec<BoundaryTask>,
    pub total_tasks: u32,
    pub total_filtered_tasks: u32,
} 