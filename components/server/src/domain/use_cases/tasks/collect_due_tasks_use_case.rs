use crate::domain::{
    entities::task::{from_boundary, save_task_aggregate, TaskAggregate},
    ports::{task_repository::TaskRepository, task_run_repository::TaskRunRepository},
};

use anyhow::Context;
use chrono::Utc;
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::{error, info};

#[derive(Clone)]
pub struct CollectDueTasksUseCase<TR, TRR> {
    pub task_repository: TR,
    pub task_run_repository: TRR,
    pub select_limit: u32,
}

impl<TR, TRR> CollectDueTasksUseCase<TR, TRR>
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

        if n_tasks == 0 {
            info!("No task will be spawned. You need to call the `run collect-due-tasks` command manually to collect due tasks");
            return join_set;
        }

        for _ in 0..n_tasks {
            let mut interval = tokio::time::interval(delay_between_two_executions);
            let executor = self.clone();

            join_set.spawn(async move {
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            match executor.collect_due_tasks().await {
                                Ok(due_tasks) if due_tasks > 0 => {
                                    info!(due_tasks, "Collected {} due tasks", due_tasks);
                                }
                                Err(e) => {
                                    error!(error = ?e, "Failed to collect due tasks")
                                }
                                Ok(_) => {}
                            }
                        }
                        _ = tokio::signal::ctrl_c() => {
                            info!("Shutting down due tasks collector task");
                            break;
                        }
                    }
                }
            });
        }

        join_set
    }

    pub async fn collect_due_tasks(&self) -> anyhow::Result<usize> {
        let mut transaction = self.task_repository.begin_transaction().await?;

        let now = Utc::now();

        let task_aggregates: Vec<TaskAggregate> = self
            .task_repository
            .list_next_due_tasks(&mut transaction, now, self.select_limit)
            .await
            .context("Failed to get due tasks from the database")?
            .into_iter()
            .map(|task| from_boundary(task, None))
            .collect::<anyhow::Result<Vec<_>>>()
            .context("Failed to convert due tasks from boundaries to task aggregates")?;
        let task_aggregates_len = task_aggregates.len();

        // turn every task aggregate into a failing one and save it
        for aggregate in task_aggregates {
            let due_aggregate = match aggregate {
                TaskAggregate::Healthy(agg) => agg.mark_due(now),
                TaskAggregate::Failing(agg) => agg.mark_due(now),
                TaskAggregate::Absent(agg) => agg.mark_due(now),
                _ => continue,
            }.context("Failed to mark task aggregate as due. This is likely a bug in the SQL query used to retrieve aggregates")?;

            save_task_aggregate(
                &self.task_repository,
                &self.task_run_repository,
                &mut transaction,
                TaskAggregate::Due(due_aggregate),
            )
            .await
            .context("Failed to save task aggregate")?;
        }

        self.task_repository
            .commit_transaction(transaction)
            .await
            .context("Failed to commit transaction")?;

        Ok(task_aggregates_len)
    }
}
