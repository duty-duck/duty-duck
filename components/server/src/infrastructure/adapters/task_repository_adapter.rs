use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::{
    entities::task::{BoundaryTask, TaskStatus, TaskUserId},
    ports::{
        task_repository::{ListTasksOpts, ListTasksOutput, TaskRepository},
        transactional_repository::TransactionalRepository,
    },
    use_cases::{shared::OrderDirection, tasks::OrderTasksBy},
};
use anyhow::*;

#[derive(Clone)]
pub struct TaskRepositoryAdapter {
    pub pool: PgPool,
}

crate::postgres_transactional_repo!(TaskRepositoryAdapter);

#[async_trait]
impl TaskRepository for TaskRepositoryAdapter {
    async fn get_task_by_uuid(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: Uuid,
    ) -> anyhow::Result<Option<BoundaryTask>> {
        let record = sqlx::query!(
            "SELECT *
            FROM tasks 
            WHERE 
                organization_id = $1 AND id = $2",
            organization_id,
            task_id,
        )
        .fetch_optional(transaction.as_mut())
        .await
        .with_context(|| "Failed to get task from database")?;

        let task = record.map(|row| BoundaryTask {
            organization_id: row.organization_id,
            id: row.id,
            user_id: TaskUserId::new(row.user_id).expect("Invalid task ID in database"),
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
            metadata: row.metadata.into(),
            schedule_timezone: row.schedule_timezone,
            email_notification_enabled: row.email_notification_enabled,
            push_notification_enabled: row.push_notification_enabled,
            sms_notification_enabled: row.sms_notification_enabled,
        });

        Ok(task)
    }

    async fn get_task_by_user_id(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_id: &TaskUserId,
    ) -> anyhow::Result<Option<BoundaryTask>> {
        let record = sqlx::query!(
            "SELECT *
            FROM tasks 
            WHERE 
                organization_id = $1 AND user_id = $2 AND status != $3",
            organization_id,
            task_id.as_str(),
            // This function should only be used to get active tasks, so we filter out archived tasks
            // This restriction also ensures that we take advantage of the partial index in the database to speed up the query
            TaskStatus::Archived as i16,
        )
        .fetch_optional(transaction.as_mut())
        .await
        .with_context(|| "Failed to get task from database")?;

        let task = record.map(|row| BoundaryTask {
            organization_id: row.organization_id,
            id: row.id,
            user_id: TaskUserId::new(row.user_id).expect("Invalid task ID in database"),
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
            metadata: row.metadata.into(),
            schedule_timezone: row.schedule_timezone,
            email_notification_enabled: row.email_notification_enabled,
            push_notification_enabled: row.push_notification_enabled,
            sms_notification_enabled: row.sms_notification_enabled,
        });

        Ok(task)
    }

    async fn list_tasks<'a>(
        &self,
        organization_id: Uuid,
        opts: ListTasksOpts<'a>,
    ) -> anyhow::Result<ListTasksOutput> {
        let mut tx = self.begin_transaction().await?;
        let query = format!("%{}%", opts.query);
        let statuses = opts
            .include_statuses
            .iter()
            .map(|s| *s as i32)
            .collect::<Vec<_>>();
        let metadata_filter = serde_json::to_value(opts.metadata_filter.items)?;

        let rows = sqlx::query(&format!(
            r#"
            WITH filter_conditions AS (
                SELECT 
                    key,
                    jsonb_array_elements_text(value) as filter_value
                FROM jsonb_each($6::jsonb) -- Replace with your filter object
            )

            SELECT *, COUNT(*) OVER() as "filtered_count!" 
            FROM tasks t
            WHERE organization_id = $1

            -- Filter by status
            AND ($2::integer[] = '{{}}' OR status = ANY($2))

            -- Filter by name or description
            AND ($3 = '' OR name ILIKE $3 OR description ILIKE $3)

            -- filter by metadata
            AND (
                $6::jsonb = '{{}}'::jsonb OR
                NOT EXISTS (
                    SELECT 1 FROM filter_conditions fc
                    WHERE NOT EXISTS (
                        SELECT 1 FROM jsonb_each(t.metadata->'records') m
                        WHERE m.key = fc.key
                        -- the braces are escaped here because we are in a format! macro
                        AND (m.value #>> '{{}}') = fc.filter_value
                    )
                )
            )

            -- Order by the chosen column and direction
            ORDER BY {} {}
            -- Limit and offset
            LIMIT $4 OFFSET $5
            "#,
            match opts.order_by {
                OrderTasksBy::CreatedAt => "created_at",
                OrderTasksBy::Name => "name",
                OrderTasksBy::LastStatusChangeAt => "last_status_change_at",
            },
            match opts.order_direction {
                OrderDirection::Asc => "ASC",
                OrderDirection::Desc => "DESC",
            }
        ))
        // $1: organization_id
        .bind(organization_id)
        // $2: statuses
        .bind(&statuses)
        // $3: query
        .bind(&query)
        // $4: limit
        .bind(opts.limit as i64)
        // $5: offset
        .bind(opts.offset as i64)
        // $6: metadata filter
        .bind(&metadata_filter)
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
            .map(|row| row.get::<i64, _>("filtered_count"))
            .unwrap_or_default();

        let tasks = rows
            .into_iter()
            .map(|row| BoundaryTask {
                organization_id: row.get("organization_id"),
                id: row.get("id"),
                user_id: TaskUserId::new(row.get::<String, _>("id"))
                    .expect("Invalid task ID in database"),
                name: row.get("name"),
                description: row.get("description"),
                status: row.get::<i16, _>("status").into(),
                previous_status: row
                    .get::<Option<i16>, _>("previous_status")
                    .map(|s| s.into()),
                last_status_change_at: row.get("last_status_change_at"),
                cron_schedule: row.get("cron_schedule"),
                next_due_at: row.get("next_due_at"),
                start_window_seconds: row.get("start_window_seconds"),
                lateness_window_seconds: row.get("lateness_window_seconds"),
                heartbeat_timeout_seconds: row.get("heartbeat_timeout_seconds"),
                created_at: row.get("created_at"),
                metadata: row.get::<Option<serde_json::Value>, _>("metadata").into(),
                schedule_timezone: row.get("schedule_timezone"),
                email_notification_enabled: row.get("email_notification_enabled"),
                push_notification_enabled: row.get("push_notification_enabled"),
                sms_notification_enabled: row.get("sms_notification_enabled"),
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
    ) -> anyhow::Result<TaskUserId> {
        sqlx::query!(
            r#"
            INSERT INTO tasks (
                organization_id, 
                id, 
                user_id,
                name, 
                description, 
                status,
                previous_status, 
                cron_schedule, 
                schedule_timezone,
                next_due_at,
                start_window_seconds, 
                lateness_window_seconds,
                heartbeat_timeout_seconds,
                last_status_change_at,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (organization_id, id) DO UPDATE SET
                name = $4,
                description = $5,
                status = $6,
                previous_status = $7,
                cron_schedule = $8,
                schedule_timezone = $9,
                next_due_at = $10,
                start_window_seconds = $11,
                lateness_window_seconds = $12,
                heartbeat_timeout_seconds = $13,
                last_status_change_at = $14,
                metadata = $15
            "#,
            task.organization_id,                   // $1
            task.id,                                // $2
            task.user_id.as_str(),                  // $3
            task.name,                              // $4
            task.description,                       // $5
            task.status as i16,                     // $6
            task.previous_status.map(|s| s as i16), // $7
            task.cron_schedule,                     // $8
            task.schedule_timezone,                 // $9
            task.next_due_at,                       // $10
            task.start_window_seconds,              // $11
            task.lateness_window_seconds,           // $12
            task.heartbeat_timeout_seconds,         // $13
            task.last_status_change_at,             // $14
            serde_json::to_value(task.metadata)?,   // $15
        )
        .execute(transaction.as_mut())
        .await?;

        Ok(task.user_id)
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
            WHERE
                cron_schedule IS NOT NULL
                AND $1::timestamptz >= next_due_at
                AND status != $2 -- status is not due 
                AND status != $3 -- status is not running
                AND status != $4 -- status is not absent
            LIMIT $5",
            now,
            TaskStatus::Due as i16,
            TaskStatus::Running as i16,
            TaskStatus::Absent as i16,
            limit as i64
        )
        .fetch_all(transaction.as_mut())
        .await?;

        let tasks = rows
            .into_iter()
            .map(|row| BoundaryTask {
                organization_id: row.organization_id,
                id: row.id,
                user_id: TaskUserId::new(row.user_id).expect("Invalid task ID in database"),
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
                metadata: row.metadata.into(),
                schedule_timezone: row.schedule_timezone,
                email_notification_enabled: row.email_notification_enabled,
                push_notification_enabled: row.push_notification_enabled,
                sms_notification_enabled: row.sms_notification_enabled,
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
                WHERE 
                    cron_schedule IS NOT NULL
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
                id: row.id,
                user_id: TaskUserId::new(row.user_id).expect("Invalid task ID in database"),
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
                metadata: row.metadata.into(),
                schedule_timezone: row.schedule_timezone,
                email_notification_enabled: row.email_notification_enabled,
                push_notification_enabled: row.push_notification_enabled,
                sms_notification_enabled: row.sms_notification_enabled,
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
            WHERE 
                cron_schedule IS NOT NULL
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
                id: row.id,
                user_id: TaskUserId::new(row.user_id).expect("Invalid task ID in database"),
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
                metadata: row.metadata.into(),
                schedule_timezone: row.schedule_timezone,
                email_notification_enabled: row.email_notification_enabled,
                push_notification_enabled: row.push_notification_enabled,
                sms_notification_enabled: row.sms_notification_enabled,
            })
            .collect();

        Ok(tasks)
    }
}
