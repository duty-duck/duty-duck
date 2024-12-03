use crate::domain::{
    entities::task::{from_boundary, save_task_aggregate, RunningTaskAggregate, TaskAggregate},
    ports::{task_repository::TaskRepository, task_run_repository::TaskRunRepository},
};

use anyhow::Context;
use chrono::Utc;
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::{error, info};

#[derive(Clone)]
pub struct CollectDeadTaskRunsUseCase<TR, TRR> {
    pub task_repository: TR,
    pub task_run_repository: TRR,
    pub select_limit: u32,
}

impl<TR, TRR> CollectDeadTaskRunsUseCase<TR, TRR>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
{
    pub fn spawn_tasks(
        &self,
        n_tasks: usize,
        delay_between_two_executions: Duration,
    ) -> JoinSet<()> {
        let mut join_set = JoinSet::new();

        for _ in 0..n_tasks {
            let mut interval = tokio::time::interval(delay_between_two_executions);
            let executor = self.clone();

            join_set.spawn(async move {
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            match executor.collect_dead_task_runs().await {
                                Ok(dead_task_runs) if dead_task_runs > 0 => {
                                    info!(dead_task_runs, "Collected {} dead task runs", dead_task_runs);
                                }
                                Err(e) => {
                                    error!(error = ?e, "Failed to clear dead task runs")
                                }
                                Ok(_) => {}
                            }
                        }
                        _ = tokio::signal::ctrl_c() => {
                            info!("Shutting down dead task runs collector task");
                            break;
                        }
                    }
                }
            });
        }

        join_set
    }

    async fn collect_dead_task_runs(&self) -> anyhow::Result<usize> {
        let mut transaction = self.task_repository.begin_transaction().await?;
        let now = Utc::now();

        let task_aggregates: Vec<TaskAggregate> = self
            .task_run_repository
            .list_dead_task_runs(&mut transaction, now, self.select_limit)
            .await
            .context("Failed to get dead task runs from the database")?
            .into_iter()
            .map(|(task, task_run)| from_boundary(task, Some(task_run)))
            .collect::<anyhow::Result<Vec<_>>>()
            .context("Failed to convert dead task runs from boundaries to task aggregates")?;

        let running_task_aggregates: Vec<RunningTaskAggregate> = task_aggregates
            .into_iter()
            .map(|agg| match agg {
                TaskAggregate::Running(agg) => Ok(agg),
                _ => Err(anyhow::anyhow!(
                    "Found a non running task aggregate. This is likely a bug in the SQL query used to retrieve aggregates"
                )),
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let running_task_aggregates_len = running_task_aggregates.len();

        // turn every running task aggregate into a failing one and save it
        for running_task_aggregate in running_task_aggregates {
            let failing_aggregate = running_task_aggregate.mark_dead(now).context("Failed to mark running task aggregate as dead. This is likely a bug in the SQL query used to retrieve aggregates")?;

            save_task_aggregate(
                &self.task_repository,
                &self.task_run_repository,
                &mut transaction,
                TaskAggregate::Failing(failing_aggregate),
            )
            .await
            .context("Failed to save task aggregate")?;
        }

        self.task_repository
            .commit_transaction(transaction)
            .await
            .context("Failed to commit transaction")?;

        Ok(running_task_aggregates_len)
    }
}
