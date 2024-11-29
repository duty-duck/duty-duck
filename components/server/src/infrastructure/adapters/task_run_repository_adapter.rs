use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    entities::{
        task::TaskId,
        task_run::{BoundaryTaskRun, TaskRunStatus},
    },
    ports::{
        task_run_repository::{ListTaskRunsOpts, TaskRunRepository},
        transactional_repository::TransactionalRepository,
    },
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
    ) -> anyhow::Result<Vec<BoundaryTaskRun>> {
        let statuses = opts
            .include_statuses
            .iter()
            .map(|s| *s as i16)
            .collect::<Vec<_>>();

        let rows = sqlx::query_as!(
            BoundaryTaskRun,
            r#"
            SELECT *
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

        Ok(rows)
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

    async fn create_task_run(
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

    async fn update_task_run(
        &self,
        transaction: &mut Self::Transaction,
        task_run: BoundaryTaskRun,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE task_runs SET
                status = $1,
                completed_at = $2,
                exit_code = $3,
                error_message = $4,
                last_heartbeat_at = $5
            WHERE organization_id = $6
            AND task_id = $7
            AND started_at = $8
            "#,
            task_run.status as i16,
            task_run.completed_at,
            task_run.exit_code,
            task_run.error_message,
            task_run.last_heartbeat_at,
            task_run.organization_id,
            task_run.task_id.as_str(),
            task_run.started_at,
        )
        .execute(transaction.as_mut())
        .await
        .context("Failed to update task run")?;

        Ok(())
    }
} 