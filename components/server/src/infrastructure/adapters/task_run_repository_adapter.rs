use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    entities::{
        task::BoundaryTask,
        task_run::{BoundaryTaskRun, TaskRunStatus},
    },
    ports::task_run_repository::{ListTaskRunsOpts, ListTaskRunsOutput, TaskRunRepository},
};
use anyhow::Context;

#[derive(Clone)]
pub struct TaskRunRepositoryAdapter {
    pub pool: PgPool,
}

impl TaskRunRepositoryAdapter {
    pub async fn create_task_run_partition_for_month(&self) -> anyhow::Result<()> {
        sqlx::query!("SELECT create_task_runs_partition_for_month()")
            .execute(&self.pool)
            .await
            .context("Failed to create task run partition for month")?;
        sqlx::query!("SELECT create_task_run_events_partition_for_month()")
            .execute(&self.pool)
            .await
            .context("Failed to create task run events partition for month")?;
        Ok(())
    }
}

crate::postgres_transactional_repo!(TaskRunRepositoryAdapter);

#[async_trait]
impl TaskRunRepository for TaskRunRepositoryAdapter {
    async fn list_task_runs<'a>(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        opts: ListTaskRunsOpts<'a>,
    ) -> anyhow::Result<ListTaskRunsOutput> {
        let statuses = opts
            .include_statuses
            .iter()
            .map(|s| *s as i16)
            .collect::<Vec<_>>();

        let rows = sqlx::query!(
            r#"
            SELECT *, COUNT(*) OVER () as "filtered_count!"
            FROM task_runs 
            WHERE 
                organization_id = $1
                AND task_id = $2
                AND ($3::smallint[] = '{}' OR status = ANY($3))
            ORDER BY started_at DESC
            LIMIT $4 OFFSET $5
            "#,
            organization_id,
            opts.task_id,
            &statuses,
            opts.limit as i64,
            opts.offset as i64,
        )
        .fetch_all(transaction.as_mut())
        .await
        .context("Failed to list task runs")?;

        let total_count = sqlx::query!(
            "SELECT count(*) FROM task_runs 
             WHERE 
                organization_id = $1 AND task_id = $2",
            organization_id,
            opts.task_id,
        )
        .fetch_one(transaction.as_mut())
        .await?
        .count
        .unwrap_or_default();

        let total_filtered_count = rows
            .first()
            .map(|row| row.filtered_count)
            .unwrap_or_default();

        let task_runs = rows
            .into_iter()
            .map(|r| BoundaryTaskRun {
                organization_id: r.organization_id,
                id: r.id,
                task_id: r.task_id,
                status: r.status.into(),
                started_at: r.started_at,
                updated_at: r.updated_at,
                completed_at: r.completed_at,
                exit_code: r.exit_code,
                error_message: r.error_message,
                last_heartbeat_at: r.last_heartbeat_at,
                heartbeat_timeout_seconds: r.heartbeat_timeout_seconds,
                metadata: r.metadata.into(),
            })
            .collect::<Vec<_>>();

        Ok(ListTaskRunsOutput {
            runs: task_runs,
            total_runs: total_count as u32,
            total_filtered_runs: total_filtered_count as u32,
        })
    }

    async fn get_task_run(
        &self,
        transaction: &mut Self::Transaction,
        organization_id: Uuid,
        task_run_id: Uuid,
    ) -> anyhow::Result<Option<BoundaryTaskRun>> {
        sqlx::query_as!(
            BoundaryTaskRun,
            r#"
            SELECT *
            FROM task_runs
            WHERE 
                organization_id = $1
                AND id = $2
            "#,
            organization_id,
            task_run_id,
        )
        .fetch_optional(transaction.as_mut())
        .await
        .context("Failed to get task run")
    }

    async fn upsert_task_run(
        &self,
        transaction: &mut Self::Transaction,
        task_run: BoundaryTaskRun,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO task_runs (
                organization_id,
                id,
                task_id,
                status,
                started_at,
                completed_at,
                exit_code,
                error_message,
                last_heartbeat_at,
                heartbeat_timeout_seconds
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (organization_id, task_id, started_at) DO UPDATE SET
                status = $4,
                completed_at = $6,
                exit_code = $7,
                error_message = $8,
                last_heartbeat_at = $9,
                heartbeat_timeout_seconds = $10
            "#,
            task_run.organization_id,           // $1
            task_run.id,                        // $2
            task_run.task_id,                   // $3
            task_run.status as i16,             // $4
            task_run.started_at,                // $5
            task_run.completed_at,              // $6
            task_run.exit_code,                 // $7
            task_run.error_message,             // $8
            task_run.last_heartbeat_at,         // $9
            task_run.heartbeat_timeout_seconds, // $10
        )
        .execute(transaction.as_mut())
        .await
        .context("Failed to create task run")?;

        Ok(())
    }

    /// List task runs that should transition to dead, along with their respective tasks
    async fn list_dead_task_runs(
        &self,
        transaction: &mut Self::Transaction,
        now: DateTime<Utc>,
        limit: u32,
    ) -> anyhow::Result<Vec<(BoundaryTask, BoundaryTaskRun)>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                tasks.status as "task_status!",
                tasks.name as "task_name!",
                tasks.user_id as "task_user_id!",
                tasks.description as "task_description",
                tasks.previous_status as "task_previous_status",
                tasks.last_status_change_at as "task_last_status_change_at",
                tasks.cron_schedule as "task_cron_schedule",
                tasks.schedule_timezone as "task_schedule_timezone",
                tasks.next_due_at as "task_next_due_at",
                tasks.start_window_seconds as "task_start_window_seconds",
                tasks.lateness_window_seconds as "task_lateness_window_seconds",
                tasks.heartbeat_timeout_seconds as "task_heartbeat_timeout_seconds",
                tasks.created_at as "task_created_at",
                tasks.metadata as "task_metadata",
                tasks.email_notification_enabled as "task_email_notification_enabled",
                tasks.push_notification_enabled as "task_push_notification_enabled",
                tasks.sms_notification_enabled as "task_sms_notification_enabled",
                task_runs.*
            FROM task_runs
            INNER JOIN tasks ON task_runs.organization_id = tasks.organization_id AND task_runs.task_id = tasks.id
            WHERE 
                (task_runs.last_heartbeat_at < ($1::timestamptz - INTERVAL '1 second' * task_runs.heartbeat_timeout_seconds)) 
                AND task_runs.status = $2
            ORDER BY task_runs.last_heartbeat_at ASC
            LIMIT $3
            "#,
            now,
            TaskRunStatus::Running as i16,
            limit as i64,
        )
        .fetch_all(transaction.as_mut())
        .await
        .context("Failed to list dead task runs")?;

        let rows = rows
            .into_iter()
            .map(|r| {
                let task = BoundaryTask {
                    id: r.task_id,
                    user_id: r.task_user_id.into(),
                    status: r.task_status.into(),
                    organization_id: r.organization_id,
                    name: r.task_name,
                    description: r.task_description,
                    previous_status: r.task_previous_status.map(|s| s.into()),
                    last_status_change_at: r.task_last_status_change_at,
                    cron_schedule: r.task_cron_schedule,
                    next_due_at: r.task_next_due_at,
                    start_window_seconds: r.task_start_window_seconds,
                    lateness_window_seconds: r.task_lateness_window_seconds,
                    heartbeat_timeout_seconds: r.task_heartbeat_timeout_seconds,
                    created_at: r.task_created_at,
                    metadata: r.task_metadata.into(),
                    schedule_timezone: r.task_schedule_timezone,
                    email_notification_enabled: r.task_email_notification_enabled,
                    push_notification_enabled: r.task_push_notification_enabled,
                    sms_notification_enabled: r.task_sms_notification_enabled,
                };

                let task_run = BoundaryTaskRun {
                    organization_id: r.organization_id,
                    id: r.id,
                    task_id: r.task_id,
                    status: r.status.into(),
                    started_at: r.started_at,
                    updated_at: r.updated_at,
                    completed_at: r.completed_at,
                    exit_code: r.exit_code,
                    error_message: r.error_message,
                    last_heartbeat_at: r.last_heartbeat_at,
                    heartbeat_timeout_seconds: r.heartbeat_timeout_seconds,
                    metadata: r.metadata.into(),
                };
                (task, task_run)
            })
            .collect::<Vec<_>>();

        Ok(rows)
    }
}
