use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::{
    task::TaskId,
    task_run::{BoundaryTaskRun, TaskRunStatus},
};

use super::transactional_repository::TransactionalRepository;

#[async_trait]
pub trait TaskRunRepository: TransactionalRepository {
    /// List the task runs for an organization
    async fn list_task_runs<'a>(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        opts: ListTaskRunsOpts<'a>,
    ) -> anyhow::Result<Vec<BoundaryTaskRun>>;

    /// Get a single task run
    async fn get_task_run(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: TaskId,
        started_at: DateTime<Utc>,
    ) -> anyhow::Result<Option<BoundaryTaskRun>>;

    /// Creates a new task run
    async fn create_task_run(
        &self,
        transaction: &mut Self::Transaction,
        task_run: BoundaryTaskRun,
    ) -> anyhow::Result<()>;

    /// Updates an existing task run
    async fn update_task_run(
        &self,
        transaction: &mut Self::Transaction,
        task_run: BoundaryTaskRun,
    ) -> anyhow::Result<()>;
}

#[derive(Clone, Debug)]
pub struct ListTaskRunsOpts<'a> {
    pub task_id: &'a TaskId,
    pub include_statuses: &'a [TaskRunStatus],
    pub limit: u32,
    pub offset: u32,
}
