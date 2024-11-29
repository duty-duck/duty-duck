use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    entities::{
        task::TaskId,
        task_run::{BoundaryTaskRun, TaskRunStatus},
    },
    ports::{
        task_run_repository::{TaskRunRepository, ListTaskRunsOpts},
        transactional_repository::{TransactionMock, TransactionalRepository},
    },
};

#[derive(Clone)]
pub struct TaskRunRepositoryMock {
    pub state: Arc<Mutex<Vec<BoundaryTaskRun>>>,
}

impl TaskRunRepositoryMock {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl TransactionalRepository for TaskRunRepositoryMock {
    type Transaction = TransactionMock;

    async fn begin_transaction(&self) -> anyhow::Result<Self::Transaction> {
        Ok(TransactionMock)
    }

    async fn commit_transaction(&self, _transaction: Self::Transaction) -> anyhow::Result<()> {
        Ok(())
    }

    async fn rollback_transaction(&self, _transaction: Self::Transaction) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl TaskRunRepository for TaskRunRepositoryMock {
    async fn list_task_runs<'a>(
        &self,
        _transaction: &mut Self::Transaction,
        organization_id: Uuid,
        opts: ListTaskRunsOpts<'a>,
    ) -> anyhow::Result<Vec<BoundaryTaskRun>> {
        let state = self.state.lock().await;
        
        let filtered_runs: Vec<BoundaryTaskRun> = state
            .iter()
            .filter(|r| r.organization_id == organization_id && r.task_id == *opts.task_id)
            .filter(|r| opts.include_statuses.is_empty() || opts.include_statuses.contains(&r.status))
            .cloned()
            .collect();

        let start = opts.offset as usize;
        let end = (opts.offset + opts.limit) as usize;
        Ok(filtered_runs[start.min(filtered_runs.len())..end.min(filtered_runs.len())].to_vec())
    }

    async fn get_task_run(
        &self,
        _transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: TaskId,
        started_at: DateTime<Utc>,
    ) -> anyhow::Result<Option<BoundaryTaskRun>> {
        let state = self.state.lock().await;
        Ok(state
            .iter()
            .find(|r| {
                r.organization_id == organization_id 
                && r.task_id == task_id 
                && r.started_at == started_at
            })
            .cloned())
    }

    async fn create_task_run(
        &self,
        _transaction: &mut Self::Transaction,
        task_run: BoundaryTaskRun,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        state.push(task_run);
        Ok(())
    }

    async fn update_task_run(
        &self,
        _transaction: &mut Self::Transaction,
        task_run: BoundaryTaskRun,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        if let Some(idx) = state.iter().position(|r| {
            r.organization_id == task_run.organization_id 
            && r.task_id == task_run.task_id 
            && r.started_at == task_run.started_at
        }) {
            state[idx] = task_run;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task_run(org_id: Uuid, task_id: &str, status: TaskRunStatus) -> BoundaryTaskRun {
        BoundaryTaskRun {
            organization_id: org_id,
            task_id: TaskId::new(task_id.to_string()).expect("Valid test task ID"),
            status,
            started_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            exit_code: None,
            error_message: None,
            last_heartbeat_at: None,
        }
    }

    #[tokio::test]
    async fn test_create_task_run_updates_state() -> anyhow::Result<()> {
        let repo = TaskRunRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        let task_run = create_test_task_run(org_id, "test-task", TaskRunStatus::Running);
        repo.create_task_run(&mut tx, task_run.clone()).await?;

        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);
        assert_eq!(state[0].organization_id, org_id);
        assert_eq!(state[0].task_id.as_str(), "test-task");
        assert_eq!(state[0].status, TaskRunStatus::Running);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_task_runs_with_status_filter() -> anyhow::Result<()> {
        let repo = TaskRunRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        // Create task runs with different statuses
        let task_runs = vec![
            create_test_task_run(org_id, "test-task", TaskRunStatus::Running),
            create_test_task_run(org_id, "test-task", TaskRunStatus::Finished),
            create_test_task_run(org_id, "test-task", TaskRunStatus::Failed),
        ];

        for task_run in task_runs {
            repo.create_task_run(&mut tx, task_run).await?;
        }

        let result = repo.list_task_runs(
            &mut tx,
            org_id,
            ListTaskRunsOpts {
                task_id: &TaskId::new("test-task".to_string()).unwrap(),
                include_statuses: &[TaskRunStatus::Running],
                limit: 10,
                offset: 0,
            },
        ).await?;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status, TaskRunStatus::Running);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_task_run() -> anyhow::Result<()> {
        let repo = TaskRunRepositoryMock::new();
        let org_id = Uuid::new_v4();
        let mut tx = repo.begin_transaction().await?;

        let mut task_run = create_test_task_run(org_id, "test-task", TaskRunStatus::Running);
        repo.create_task_run(&mut tx, task_run.clone()).await?;

        task_run.status = TaskRunStatus::Finished;
        task_run.completed_at = Some(Utc::now());
        task_run.exit_code = Some(0);

        repo.update_task_run(&mut tx, task_run.clone()).await?;

        let state = repo.state.lock().await;
        assert_eq!(state[0].status, TaskRunStatus::Finished);
        assert!(state[0].completed_at.is_some());
        assert_eq!(state[0].exit_code, Some(0));

        Ok(())
    }
} 