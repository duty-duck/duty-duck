use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    entities::{task::TaskId, task_run::BoundaryTaskRun},
    ports::task_run_repository::{ListTaskRunsOpts, ListTaskRunsOutput, TaskRunRepository},
};
use anyhow::Context;

#[derive(Clone)]
pub struct TaskRunRepositoryAdapter {
    pub pool: PgPool,
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
            WHERE organization_id = $1
            AND task_id = $2
            AND ($3::smallint[] IS NULL OR status = ANY($3))
            ORDER BY started_at DESC
            LIMIT $4 OFFSET $5
            "#,
            organization_id,
            opts.task_id.as_str(),
            &statuses,
            opts.limit as i64,
            opts.offset as i64,
        )
        .fetch_all(transaction.as_mut())
        .await
        .context("Failed to list task runs")?;

        let total_count = sqlx::query!(
            "SELECT count(*) FROM task_runs WHERE organization_id = $1 AND task_id = $2",
            organization_id,
            opts.task_id.as_str(),
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
                task_id: r.task_id.into(),
                status: r.status.into(),
                started_at: r.started_at,
                updated_at: r.updated_at,
                completed_at: r.completed_at,
                exit_code: r.exit_code,
                error_message: r.error_message,
                last_heartbeat_at: r.last_heartbeat_at,
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
        task_id: TaskId,
        started_at: DateTime<Utc>,
    ) -> anyhow::Result<Option<BoundaryTaskRun>> {
        sqlx::query_as!(
            BoundaryTaskRun,
            r#"
            SELECT *
            FROM task_runs
            WHERE organization_id = $1
            AND task_id = $2
            AND started_at = $3
            "#,
            organization_id,
            task_id.as_str(),
            started_at,
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
                task_id,
                status,
                started_at,
                completed_at,
                exit_code,
                error_message,
                last_heartbeat_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (organization_id, task_id, started_at) DO UPDATE SET
                status = $3,
                completed_at = $5,
                exit_code = $6,
                error_message = $7,
                last_heartbeat_at = $8
            "#,
            task_run.organization_id,
            task_run.task_id.as_str(),
            task_run.status as i16,
            task_run.started_at,
            task_run.completed_at,
            task_run.exit_code,
            task_run.error_message,
            task_run.last_heartbeat_at,
        )
        .execute(transaction.as_mut())
        .await
        .context("Failed to create task run")?;

        Ok(())
    }
    
}
