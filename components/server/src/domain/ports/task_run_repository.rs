use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::{
    task::{BoundaryTask, TaskId},
    task_run::{BoundaryTaskRun, TaskRunStatus},
};

use super::transactional_repository::TransactionalRepository;

#[async_trait]
pub trait TaskRunRepository: TransactionalRepository + Clone + Send + Sync + 'static {
    /// List the task runs for an organization
    async fn list_task_runs<'a>(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        opts: ListTaskRunsOpts<'a>,
    ) -> anyhow::Result<ListTaskRunsOutput>;

    async fn get_latest_task_run(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: &TaskId,
        statuses: &[TaskRunStatus],
    ) -> anyhow::Result<Option<BoundaryTaskRun>> {
        Ok(self
            .list_task_runs(
                transaction,
                organization_id,
                ListTaskRunsOpts {
                    task_id,
                    include_statuses: statuses,
                    limit: 1,
                    offset: 0,
                },
            )
            .await?
            .runs
            .into_iter()
            .next())
    }

    /// Get a single task run
    async fn get_task_run(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: &TaskId,
        started_at: DateTime<Utc>,
    ) -> anyhow::Result<Option<BoundaryTaskRun>>;

    /// Creates or updates an existing task run
    async fn upsert_task_run(
        &self,
        transaction: &mut Self::Transaction,
        task_run: BoundaryTaskRun,
    ) -> anyhow::Result<()>;

    /// List task runs that should transition to dead, along with their respective tasks
    async fn list_dead_task_runs(
        &self,
        transaction: &mut Self::Transaction,
        now: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<(BoundaryTask, BoundaryTaskRun)>>;
}

#[derive(Clone, Debug)]
pub struct ListTaskRunsOpts<'a> {
    pub task_id: &'a TaskId,
    pub include_statuses: &'a [TaskRunStatus],
    pub limit: u32,
    pub offset: u32,
}

pub struct ListTaskRunsOutput {
    pub runs: Vec<BoundaryTaskRun>,
    pub total_filtered_runs: u32,
    pub total_runs: u32,
}
