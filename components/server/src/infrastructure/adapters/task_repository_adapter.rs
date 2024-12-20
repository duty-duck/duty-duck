use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    entities::task::{BoundaryTask, TaskId, TaskStatus},
    ports::{
        task_repository::{ListTasksOutput, TaskRepository},
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
        task_id: &TaskId,
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
            AND ($2::integer[] = '{}' OR status = ANY($2))
            AND ($3 = '' OR name ILIKE $3 OR description ILIKE $3)
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

    async fn upsert_task(
        &self,
        transaction: &mut Self::Transaction,
        task: BoundaryTask,
    ) -> anyhow::Result<TaskId> {
        sqlx::query!(
            r#"
            INSERT INTO tasks (
                organization_id, 
                id, 
                name, 
                description, 
                status,
                previous_status, 
                cron_schedule, 
                next_due_at,
                start_window_seconds, 
                lateness_window_seconds,
                heartbeat_timeout_seconds,
                last_status_change_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (organization_id, id) DO UPDATE SET
                name = $3,
                description = $4,
                status = $5,
                cron_schedule = $7,
                next_due_at = $8,
                start_window_seconds = $9,
                lateness_window_seconds = $10,
                heartbeat_timeout_seconds = $11,
                last_status_change_at = $12
            "#,
            task.organization_id, // $1
            task.id.as_str(), // $2
            task.name, // $3
            task.description, // $4
            task.status as i16, // $5
            task.status as i16, // $6 - Initial previous_status is same as status
            task.cron_schedule, // $7
            task.next_due_at, // $8
            task.start_window_seconds, // $9
            task.lateness_window_seconds, // $10
            task.heartbeat_timeout_seconds, // $11
            task.last_status_change_at, // $12
        )
        .execute(transaction.as_mut())
        .await?;

        Ok(task.id)
    }

    /// List scheduled tasks that should transition to Due
    async fn list_next_due_tasks(
        &self,
        transaction: &mut Self::Transaction,
        now: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<BoundaryTask>> {
        let rows = sqlx::query!(
            "
            SELECT * FROM tasks
            WHERE cron_schedule IS NOT NULL
            AND $1::timestamptz >= next_due_at
            AND status != $2 -- status is not due 
            AND status != $3 -- status is not running
            LIMIT $4",
            now,
            TaskStatus::Due as i16,
            TaskStatus::Running as i16,
            limit as i64
        )
        .fetch_all(transaction.as_mut())
        .await?;

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

        Ok(tasks)
    }

        /// List due tasks that should transition to Late
        async fn list_due_tasks_running_late(
            &self,
            transaction: &mut Self::Transaction,
            now: DateTime<Utc>,
            limit: u32,
        ) -> anyhow::Result<Vec<BoundaryTask>> {
            let rows = sqlx::query!(
                "
                SELECT * FROM tasks
                WHERE cron_schedule IS NOT NULL
                AND $1::timestamptz >= next_due_at + (start_window_seconds || ' seconds')::interval
                AND status = $2 -- status is due
                LIMIT $3",
                now,
                TaskStatus::Due as i16,
                limit as i64
            )
            .fetch_all(transaction.as_mut())
            .await?;
    
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
    
            Ok(tasks)
        }
    
        /// List late tasks that should transition to Absent
        async fn list_next_absent_tasks(
            &self,
            transaction: &mut Self::Transaction,
            now: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<BoundaryTask>> {
        let rows = sqlx::query!(
            "
            SELECT * FROM tasks
            WHERE cron_schedule IS NOT NULL
            AND $1::timestamptz >= next_due_at + (start_window_seconds || ' seconds')::interval + (lateness_window_seconds || ' seconds')::interval
            AND status = $2 -- status is late
            LIMIT $3",
            now,
            TaskStatus::Late as i16,
            limit as i64
        )
        .fetch_all(transaction.as_mut())
        .await?;

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

        Ok(tasks)
    }
}
