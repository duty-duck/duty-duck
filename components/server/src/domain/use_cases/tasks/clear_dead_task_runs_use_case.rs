use crate::domain::ports::{
    task_repository::TaskRepository, task_run_repository::TaskRunRepository,
};

use std::time::Duration;
use tokio::task::JoinSet;
use tracing::{info, error};

#[derive(Clone)]
pub struct ClearDeadTaskRunsUseCase<TR, TRR> {
    pub task_repository: TR,
    pub task_run_repository: TRR,
    pub select_limit: u32,
}

impl<TR, TRR> ClearDeadTaskRunsUseCase<TR, TRR>
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
                            match executor.clear_dead_task_runs().await {
                                Ok(dead_task_runs) if dead_task_runs > 0 => {
                                    info!(dead_task_runs, "Cleared {} dead task runs", dead_task_runs);
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

    async fn clear_dead_task_runs(&self) -> anyhow::Result<usize> {
        todo!()
    }
}
