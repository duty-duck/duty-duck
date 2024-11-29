use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    entities::task::{BoundaryTask, TaskId, TaskStatus},
    ports::{
        task_repository::{
            ListTasksOutput, TaskRepository, UpdateTaskStatusCommand,
        },
        transactional_repository::TransactionalRepository,
    },
};
use anyhow::*;

#[derive(Clone)]
pub struct TaskRepositoryAdapter {
    pub pool: PgPool,
}

crate::postgres_transactional_repo!(TaskRepositoryAdapter);

#[async_trait]
impl TaskRepository for TaskRepositoryAdapter {
    async fn get_task(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: TaskId,
    ) -> anyhow::Result<Option<BoundaryTask>> {
        let record = sqlx::query!(
            "SELECT *
            FROM tasks 
            WHERE organization_id = $1 AND id = $2",
            organization_id,
            task_id.as_str(),
        )
        .fetch_optional(transaction.as_mut())
        .await
        .with_context(|| "Failed to get task from database")?;

        let task = record.map(|row| BoundaryTask {
            organization_id: row.organization_id,
            id: TaskId::new(row.id).expect("Invalid task ID in database"),
            name: row.name,
            description: row.description,
            status: TaskStatus::from(row.status),
            previous_status: row.previous_status.map(TaskStatus::from),
            last_status_change_at: row.last_status_change_at,
            cron_schedule: row.cron_schedule,
            next_due_at: row.next_due_at,
            start_window_seconds: row.start_window_seconds,
            lateness_window_seconds: row.lateness_window_seconds,
            heartbeat_timeout_seconds: row.heartbeat_timeout_seconds,
            created_at: row.created_at,
        });

        Ok(task)
    }

    async fn list_tasks(
        &self,
        organization_id: Uuid,
        include_statuses: Vec<TaskStatus>,
        query: String,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<ListTasksOutput> {
        let mut tx = self.begin_transaction().await?;
        let query = format!("%{query}%");
        let statuses = include_statuses
            .into_iter()
            .map(|s| s as i32)
            .collect::<Vec<_>>();

        let rows = sqlx::query!(
            r#"
            SELECT *, COUNT(*) OVER() as "filtered_count!" 
            FROM tasks
            WHERE organization_id = $1
            AND ($2::integer[] IS NULL OR status = ANY($2))
            AND (name ILIKE $3 OR description ILIKE $3)
            ORDER BY name
            LIMIT $4 OFFSET $5
            "#,
            organization_id,
            &statuses,
            &query,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut *tx)
        .await?;

        let total_count = sqlx::query!(
            "SELECT count(*) FROM tasks WHERE organization_id = $1",
            organization_id
        )
        .fetch_one(&mut *tx)
        .await?
        .count
        .ok_or_else(|| anyhow!("Count should not be null"))?;

        let total_filtered_count = rows
            .first()
            .map(|row| row.filtered_count)
            .unwrap_or_default();

        let tasks = rows
            .into_iter()
            .map(|row| BoundaryTask {
                organization_id: row.organization_id,
                id: TaskId::new(row.id).expect("Invalid task ID in database"),
                name: row.name,
                description: row.description,
                status: TaskStatus::from(row.status),
                previous_status: row.previous_status.map(TaskStatus::from),
                last_status_change_at: row.last_status_change_at,
                cron_schedule: row.cron_schedule,
                next_due_at: row.next_due_at,
                start_window_seconds: row.start_window_seconds,
                lateness_window_seconds: row.lateness_window_seconds,
                heartbeat_timeout_seconds: row.heartbeat_timeout_seconds,
                created_at: row.created_at,
            })
            .collect();

        Ok(ListTasksOutput {
            tasks,
            total_tasks: total_count as u32,
            total_filtered_tasks: total_filtered_count as u32,
        })
    }

    async fn create_task(&self, task: BoundaryTask) -> anyhow::Result<TaskId> {
        sqlx::query!(
            r#"
            INSERT INTO tasks (
                organization_id, id, name, description, status,
                previous_status, cron_schedule, next_due_at,
                start_window_seconds, lateness_window_seconds,
                heartbeat_timeout_seconds
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            task.organization_id,
            task.id.as_str(),
            task.name,
            task.description,
            task.status as i16,
            task.status as i16,  // Initial previous_status is same as status
            task.cron_schedule,
            task.next_due_at,
            task.start_window_seconds,
            task.lateness_window_seconds,
            task.heartbeat_timeout_seconds,
        )
        .execute(&self.pool)
        .await?;

        Ok(task.id)
    }

    async fn update_task(
        &self,
        transaction: &mut Self::Transaction,
        task: BoundaryTask,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE tasks SET
                name = $1,
                description = $2,
                status = $3,
                cron_schedule = $4,
                next_due_at = $5,
                start_window_seconds = $6,
                lateness_window_seconds = $7,
                heartbeat_timeout_seconds = $8
            WHERE organization_id = $9 AND id = $10
            "#,
            task.name,
            task.description,
            task.status as i16,
            task.cron_schedule,
            task.next_due_at,
            task.start_window_seconds,
            task.lateness_window_seconds,
            task.heartbeat_timeout_seconds,
            task.organization_id,
            task.id.as_str(),
        )
        .execute(transaction.as_mut())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn update_task_status(
        &self,
        transaction: &mut Self::Transaction,
        command: UpdateTaskStatusCommand,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE tasks SET
                status = $1,
                previous_status = $2,
                last_status_change_at = $3,
                next_due_at = $4
            WHERE organization_id = $5 AND id = $6
            "#,
            command.status as i16,
            command.previous_status as i16,
            command.last_status_change_at,
            command.next_due_at,
            command.organization_id,
            command.task_id.as_str(),
        )
        .execute(transaction.as_mut())
        .await?;

        Ok(())
    }
} 