use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::{
    entities::task::{BoundaryTask, TaskStatus, TaskUserId},
    ports::{
        task_repository::{ListTasksOpts, ListTasksOutput, TaskRepository},
        transactional_repository::{TransactionMock, TransactionalRepository},
    },
};

#[derive(Clone)]
pub struct TaskRepositoryMock {
    pub state: Arc<Mutex<Vec<BoundaryTask>>>,
}

impl TaskRepositoryMock {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl TransactionalRepository for TaskRepositoryMock {
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
impl TaskRepository for TaskRepositoryMock {
    async fn get_task_by_uuid(
        &self,
        _transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: Uuid,
    ) -> anyhow::Result<Option<BoundaryTask>> {
        let state = self.state.lock().await;
        Ok(state
            .iter()
            .find(|t| t.id == task_id && t.organization_id == organization_id)
            .cloned())
    }

    async fn get_task_by_user_id(
        &self,
        _transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: &TaskUserId,
    ) -> anyhow::Result<Option<BoundaryTask>> {
        let state = self.state.lock().await;
        Ok(state
            .iter()
            .find(|t| {
                t.user_id == *task_id
                    && t.organization_id == organization_id
                    && t.status != TaskStatus::Archived
            })
            .cloned())
    }

    async fn list_tasks<'a>(
        &self,
        organization_id: Uuid,
        opts: ListTasksOpts<'a>,
    ) -> anyhow::Result<ListTasksOutput> {
        let state = self.state.lock().await;

        let total_tasks = state
            .iter()
            .filter(|t| t.organization_id == organization_id)
            .count() as u32;

        let filtered_tasks: Vec<BoundaryTask> = state
            .iter()
            .filter(|t| t.organization_id == organization_id)
            .filter(|t| {
                opts.include_statuses.is_empty() || opts.include_statuses.contains(&t.status)
            })
            .filter(|t| {
                opts.query.is_empty()
                    || t.name.to_lowercase().contains(&opts.query.to_lowercase())
                    || t.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&opts.query.to_lowercase()))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();

        let total_filtered_tasks = filtered_tasks.len() as u32;

        let start = opts.offset as usize;
        let end = (opts.offset + opts.limit) as usize;
        let tasks =
            filtered_tasks[start.min(filtered_tasks.len())..end.min(filtered_tasks.len())].to_vec();

        Ok(ListTasksOutput {
            tasks,
            total_tasks,
            total_filtered_tasks,
        })
    }

    async fn upsert_task(
        &self,
        _transaction: &mut Self::Transaction,
        task: BoundaryTask,
    ) -> anyhow::Result<TaskUserId> {
        let mut state = self.state.lock().await;

        if let Some(existing) = state
            .iter_mut()
            .find(|t| t.id == task.id && t.organization_id == task.organization_id)
        {
            *existing = task;
            Ok(existing.user_id.clone())
        } else {
            let id = task.user_id.clone();
            state.push(task);
            Ok(id)
        }
    }

    async fn list_next_due_tasks(
        &self,
        _transaction: &mut Self::Transaction,
        now: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<BoundaryTask>> {
        let state = self.state.lock().await;
        Ok(state
            .iter()
            .filter(|t| {
                t.status == TaskStatus::Due
                    && t.status != TaskStatus::Running
                    && t.next_due_at.is_some_and(|due_at| now >= due_at)
            })
            .take(limit as usize)
            .cloned()
            .collect())
    }

    async fn list_due_tasks_running_late(
        &self,
        _transaction: &mut Self::Transaction,
        now: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<BoundaryTask>> {
        let state = self.state.lock().await;
        Ok(state
            .iter()
            .filter(|t| {
                t.status == TaskStatus::Due
                    && t.next_due_at.is_some_and(|due_at| {
                        now >= due_at + Duration::from_secs(t.start_window_seconds as u64)
                    })
            })
            .take(limit as usize)
            .cloned()
            .collect())
    }

    async fn list_next_absent_tasks(
        &self,
        _transaction: &mut Self::Transaction,
        now: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<BoundaryTask>> {
        let state = self.state.lock().await;
        Ok(state
            .iter()
            .filter(|t| {
                t.status == TaskStatus::Late
                    && t.next_due_at.is_some_and(|due_at| {
                        now >= due_at
                            + Duration::from_secs(
                                t.start_window_seconds as u64 + t.lateness_window_seconds as u64,
                            )
                    })
            })
            .take(limit as usize)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entities::entity_metadata::EntityMetadata;

    use super::*;

    fn create_test_task(org_id: Uuid, name: &str, status: TaskStatus) -> BoundaryTask {
        BoundaryTask {
            organization_id: org_id,
            id: Uuid::new_v4(),
            user_id: TaskUserId::new(name.to_string()).expect("Invalid task ID"),
            name: name.to_string(),
            description: None,
            status,
            previous_status: None,
            last_status_change_at: None,
            cron_schedule: None,
            next_due_at: Some(Utc::now()),
            start_window_seconds: 300,
            lateness_window_seconds: 600,
            heartbeat_timeout_seconds: 60,
            created_at: Utc::now(),
            metadata: EntityMetadata::default(),
            schedule_timezone: None,
            email_notification_enabled: true,
            push_notification_enabled: true,
            sms_notification_enabled: false,
        }
    }

    #[tokio::test]
    async fn test_create_task_updates_state() -> anyhow::Result<()> {
        let repo = TaskRepositoryMock::new();
        let org_id = Uuid::new_v4();

        let task = create_test_task(org_id, "test-task", TaskStatus::Healthy);
        let id = repo.upsert_task(&mut TransactionMock, task).await?;

        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);

        let created_task = &state[0];
        assert_eq!(created_task.user_id, id);
        assert_eq!(created_task.organization_id, org_id);
        assert_eq!(created_task.name, "test-task");
        assert_eq!(created_task.status, TaskStatus::Healthy);

        Ok(())
    }

    // Add more tests similar to http_monitor_repository_mock tests...
}
