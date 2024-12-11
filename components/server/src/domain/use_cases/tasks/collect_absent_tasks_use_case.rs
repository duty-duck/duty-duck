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
pub struct CollectAbsentTasksUseCase<TR, TRR> {
    pub task_repository: TR,
    pub task_run_repository: TRR,
    pub select_limit: u32,
}

impl<TR, TRR> CollectAbsentTasksUseCase<TR, TRR>
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
            info!("No task will be spawned. You need to call the `run collect-absent-tasks` command manually to collect absent tasks");
            return join_set;
        }

        for _ in 0..n_tasks {
            let mut interval = tokio::time::interval(delay_between_two_executions);
            let executor = self.clone();

            join_set.spawn(async move {
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            match executor.collect_absent_tasks().await {
                                Ok(absent_tasks) if absent_tasks > 0 => {
                                    info!(absent_tasks, "Collected {} absent tasks", absent_tasks);
                                }
                                Err(e) => {
                                    error!(error = ?e, "Failed to collect absent tasks")
                                }
                                Ok(_) => {}
                            }
                        }
                        _ = tokio::signal::ctrl_c() => {
                            info!("Shutting down absent tasks collector task");
                            break;
                        }
                    }
                }
            });
        }

        join_set
    }

    pub async fn collect_absent_tasks(&self) -> anyhow::Result<usize> {
        let mut transaction = self.task_repository.begin_transaction().await?;

        let now = Utc::now();

        let task_aggregates: Vec<TaskAggregate> = self
            .task_repository
            .list_next_absent_tasks(&mut transaction, now, self.select_limit)
            .await
            .context("Failed to get absent tasks from the database")?
            .into_iter()
            .map(|task| from_boundary(task, None))
            .collect::<anyhow::Result<Vec<_>>>()
            .context("Failed to convert absent tasks from boundaries to task aggregates")?;
        let task_aggregates_len = task_aggregates.len();

        // turn every task aggregate into a failing one and save it
        for aggregate in task_aggregates {
            let absent_aggregate = match aggregate {
                TaskAggregate::Late(agg) => agg.mark_absent(now).context("Failed to mark task aggregate as absent. This is likely a bug in the SQL query used to retrieve aggregates")?,
                _ => anyhow::bail!("unexpected task aggregate type. This is likely a bug in the SQL query used to retrieve aggregates"),
            };

            save_task_aggregate(
                &self.task_repository,
                &self.task_run_repository,
                &mut transaction,
                TaskAggregate::Absent(absent_aggregate),
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
