use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::transactional_repository::TransactionalRepository;
use crate::domain::{
    entities::{
        entity_metadata::MetadataFilter,
        task::{BoundaryTask, TaskId, TaskStatus, TaskUserId},
    },
    use_cases::{shared::OrderDirection, tasks::OrderTasksBy},
};

#[async_trait]
pub trait TaskRepository: TransactionalRepository + Clone + Send + Sync + 'static {
    async fn get_task_by_id(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: &TaskId,
    ) -> anyhow::Result<Option<BoundaryTask>> {
        match task_id {
            TaskId::Uuid(uuid) => {
                self.get_task_by_uuid(transaction, organization_id, *uuid)
                    .await
            }
            TaskId::UserId(user_id) => {
                self.get_task_by_user_id(transaction, organization_id, user_id)
                    .await
            }
        }
    }

    /// Get a single task by organization ID and task uuid
    async fn get_task_by_uuid(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: Uuid,
    ) -> anyhow::Result<Option<BoundaryTask>>;

    /// Get a single task by organization ID and its user-defined ID
    /// This function can only return a task that is not archived. This is because user-defined ids are unique all active tasks within an organization,
    /// but archived tasks are allowed to have the same user-defined id. This is enforced by the database.
    /// This function will return None if the task is archived
    async fn get_task_by_user_id(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: &TaskUserId,
    ) -> anyhow::Result<Option<BoundaryTask>>;

    /// List tasks with pagination and filtering
    async fn list_tasks<'a>(
        &self,
        organization_id: Uuid,
        opts: ListTasksOpts<'a>,
    ) -> anyhow::Result<ListTasksOutput>;

    /// Create or update an existing task
    async fn upsert_task(
        &self,
        transaction: &mut Self::Transaction,
        task: BoundaryTask,
    ) -> anyhow::Result<TaskUserId>;

    /// List scheduled tasks that should transition to Due
    async fn list_next_due_tasks(
        &self,
        transaction: &mut Self::Transaction,
        now: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<BoundaryTask>>;

    /// List due tasks that should transition to Late
    async fn list_due_tasks_running_late(
        &self,
        transaction: &mut Self::Transaction,
        now: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<BoundaryTask>>;

    /// List late tasks that should transition to Absent
    async fn list_next_absent_tasks(
        &self,
        transaction: &mut Self::Transaction,
        now: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<BoundaryTask>>;
}

#[derive(Clone, Debug, Default)]
pub struct ListTasksOpts<'a> {
    pub query: &'a str,
    pub include_statuses: &'a [TaskStatus],
    pub metadata_filter: MetadataFilter,
    pub limit: u32,
    pub offset: u32,
    pub order_by: OrderTasksBy,
    pub order_direction: OrderDirection,
}

pub struct ListTasksOutput {
    pub tasks: Vec<BoundaryTask>,
    pub total_tasks: u32,
    pub total_filtered_tasks: u32,
}
