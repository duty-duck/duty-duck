use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    entities::task::{Task, TaskId, TaskStatus},
    ports::{
        task_repository::{TaskRepository, ListTasksOutput, NewTask, UpdateTaskStatusCommand},
        transactional_repository::{TransactionMock, TransactionalRepository},
    },
};

#[derive(Clone)]
pub struct TaskRepositoryMock {
    pub state: Arc<Mutex<Vec<Task>>>,
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
    async fn get_task(
        &self,
        _transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: TaskId,
    ) -> anyhow::Result<Option<Task>> {
        let state = self.state.lock().await;
        Ok(state
            .iter()
            .find(|t| t.id == task_id && t.organization_id == organization_id)
            .cloned())
    }

    async fn list_tasks(
        &self,
        organization_id: Uuid,
        include_statuses: Vec<TaskStatus>,
        query: String,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<ListTasksOutput> {
        let state = self.state.lock().await;
        
        let total_tasks = state
            .iter()
            .filter(|t| t.organization_id == organization_id)
            .count() as u32;

        let filtered_tasks: Vec<Task> = state
            .iter()
            .filter(|t| t.organization_id == organization_id)
            .filter(|t| include_statuses.is_empty() || include_statuses.contains(&t.status))
            .filter(|t| {
                query.is_empty() || 
                t.name.to_lowercase().contains(&query.to_lowercase()) ||
                t.description.as_ref().map(|d| d.to_lowercase().contains(&query.to_lowercase())).unwrap_or(false)
            })
            .cloned()
            .collect();

        let total_filtered_tasks = filtered_tasks.len() as u32;
        
        let start = offset as usize;
        let end = (offset + limit) as usize;
        let tasks = filtered_tasks[start.min(filtered_tasks.len())..end.min(filtered_tasks.len())].to_vec();

        Ok(ListTasksOutput {
            tasks,
            total_tasks,
            total_filtered_tasks,
        })
    }

    async fn create_task(&self, task: NewTask) -> anyhow::Result<TaskId> {
        let mut state = self.state.lock().await;
        
        let task = Task {
            organization_id: task.organization_id,
            id: task.id.clone(),
            name: task.name,
            description: task.description,
            status: task.status,
            previous_status: task.status,
            last_status_change_at: None,
            cron_schedule: task.cron_schedule,
            next_due_at: task.next_due_at,
            start_window_seconds: task.start_window_seconds,
            lateness_window_seconds: task.lateness_window_seconds,
            heartbeat_timeout_seconds: task.heartbeat_timeout_seconds,
            created_at: Utc::now(),
        };

        let id = task.id.clone();
        state.push(task);
        Ok(id)
    }

    async fn update_task(
        &self,
        _transaction: &mut Self::Transaction,
        id: TaskId,
        task: NewTask,
    ) -> anyhow::Result<bool> {
        let mut state = self.state.lock().await;

        if let Some(existing) = state.iter_mut().find(|t| t.id == id && t.organization_id == task.organization_id) {
            existing.name = task.name;
            existing.description = task.description;
            existing.status = task.status;
            existing.cron_schedule = task.cron_schedule;
            existing.next_due_at = task.next_due_at;
            existing.start_window_seconds = task.start_window_seconds;
            existing.lateness_window_seconds = task.lateness_window_seconds;
            existing.heartbeat_timeout_seconds = task.heartbeat_timeout_seconds;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn update_task_status(
        &self,
        _transaction: &mut Self::Transaction,
        command: UpdateTaskStatusCommand,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        
        if let Some(task) = state.iter_mut().find(|t| {
            t.id == command.task_id && t.organization_id == command.organization_id
        }) {
            task.previous_status = command.previous_status;
            task.status = command.status;
            task.last_status_change_at = command.last_status_change_at;
            task.next_due_at = command.next_due_at;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task(org_id: Uuid, name: &str, status: TaskStatus) -> NewTask {
        NewTask {
            organization_id: org_id,
            id: TaskId::new(name.to_string()).expect("Invalid task ID"),
            name: name.to_string(),
            description: None,
            status,
            cron_schedule: None,
            next_due_at: Some(Utc::now()),
            start_window_seconds: 300,
            lateness_window_seconds: 600,
            heartbeat_timeout_seconds: 60,
        }
    }

    #[tokio::test]
    async fn test_create_task_updates_state() -> anyhow::Result<()> {
        let repo = TaskRepositoryMock::new();
        let org_id = Uuid::new_v4();
        
        let task = create_test_task(org_id, "test-task", TaskStatus::Pending);
        let id = repo.create_task(task).await?;
        
        let state = repo.state.lock().await;
        assert_eq!(state.len(), 1);
        
        let created_task = &state[0];
        assert_eq!(created_task.id, id);
        assert_eq!(created_task.organization_id, org_id);
        assert_eq!(created_task.name, "test-task");
        assert_eq!(created_task.status, TaskStatus::Pending);
        
        Ok(())
    }

    // Add more tests similar to http_monitor_repository_mock tests...
} 