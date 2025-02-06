use crate::domain::{
    entities::{
        entity_metadata::EntityMetadata,
        incident::{
            IncidentCause, IncidentPriority, IncidentSource, IncidentStatus, NewIncident,
            TaskRunIncidentCause,
        },
        incident_event::{IncidentEvent, IncidentEventType},
        incident_notification::IncidentNotificationPayload,
        task::{
            from_boundary, save_task_aggregate, FailingTaskAggregate, FailingTaskRun,
            RunningTaskAggregate, TaskAggregate,
        },
        task_run::TaskRunStatus,
    },
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::IncidentRepository, task_repository::TaskRepository,
        task_run_repository::TaskRunRepository,
    },
    use_cases::incidents::{create_incident, NotificationOpts},
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::{error, info};

#[derive(Clone)]
pub struct CollectDeadTaskRunsUseCase<TR, TRR, IR, IER, INR> {
    pub task_repository: TR,
    pub task_run_repository: TRR,
    pub incident_repository: IR,
    pub incident_event_repository: IER,
    pub incident_notification_repository: INR,
    pub select_limit: u32,
}

impl<TR, TRR, IR, IER, INR> CollectDeadTaskRunsUseCase<TR, TRR, IR, IER, INR>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
    IR: IncidentRepository<Transaction = TR::Transaction>,
    IER: IncidentEventRepository<Transaction = TR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = TR::Transaction>,
{
    pub fn spawn_tasks(
        &self,
        n_tasks: usize,
        delay_between_two_executions: Duration,
    ) -> JoinSet<()> {
        let mut join_set = JoinSet::new();
        if n_tasks == 0 {
            info!("No task will be spawned. You need to call the `run collect-dead-task-runs` command manually to collect dead task runs");
            return join_set;
        }

        for _ in 0..n_tasks {
            let mut interval = tokio::time::interval(delay_between_two_executions);
            let executor = self.clone();

            join_set.spawn(async move {
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            let now = Utc::now();
                            match executor.collect_dead_task_runs(now).await {
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

    pub async fn collect_dead_task_runs(&self, now: DateTime<Utc>) -> anyhow::Result<usize> {
        let mut transaction = self.task_repository.begin_transaction().await?;

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
            self.process_dead_task_aggregate(&mut transaction, failing_aggregate, now)
                .await?;
        }

        self.task_repository
            .commit_transaction(transaction)
            .await
            .context("Failed to commit transaction")?;

        Ok(running_task_aggregates_len)
    }

    async fn process_dead_task_aggregate(
        &self,
        transaction: &mut TR::Transaction,
        dead_task_aggregate: FailingTaskAggregate,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let organization_id = *dead_task_aggregate.task().base().organization_id();
        let dead_task_run = match dead_task_aggregate.task_run() {
            FailingTaskRun::Dead(dead_task_run) => dead_task_run,
            FailingTaskRun::Failed(_) => unreachable!(),
        };

        let incident_cause = IncidentCause::TaskRunIncidentCause(TaskRunIncidentCause {
            task_id: dead_task_aggregate.task().base().user_id().clone(),
            task_run_id: *dead_task_run.id(),
            task_run_started_at: *dead_task_run.started_at(),
            task_run_finished_at: Some(*dead_task_run.completed_at()),
            task_run_status: TaskRunStatus::Dead,
        });

        let incident_id = create_incident(
            transaction,
            &self.incident_repository,
            &self.incident_event_repository,
            &self.incident_notification_repository,
            now,
            NewIncident {
                organization_id,
                created_by: None,
                status: IncidentStatus::Ongoing,
                priority: IncidentPriority::Major,
                source: IncidentSource::TaskRun {
                    id: *dead_task_run.id(),
                },
                cause: Some(incident_cause.clone()),
                metadata: EntityMetadata::default(),
            },
            Some(NotificationOpts {
                send_sms: *dead_task_aggregate.task().base().sms_notification_enabled(),
                send_push_notification: *dead_task_aggregate
                    .task()
                    .base()
                    .push_notification_enabled(),
                send_email: *dead_task_aggregate
                    .task()
                    .base()
                    .email_notification_enabled(),
                notification_payload: IncidentNotificationPayload {
                    incident_cause,
                    incident_task_id: Some(dead_task_aggregate.task().base().user_id().clone()),
                    incident_http_monitor_url: None,
                },
            }),
        )
        .await?;

        // Create additional events
        let task_run_started_event = IncidentEvent {
            organization_id,
            incident_id,
            user_id: None,
            created_at: *dead_task_run.started_at(),
            event_type: IncidentEventType::TaskRunStarted,
            event_payload: None,
        };
        let task_run_last_heartbeat_event = IncidentEvent {
            organization_id,
            incident_id,
            user_id: None,
            created_at: *dead_task_run.last_heartbeat_at(),
            event_type: IncidentEventType::TaskRunReceivedLastHeartbeat,
            event_payload: None,
        };
        let task_run_dead_event = IncidentEvent {
            organization_id,
            incident_id,
            user_id: None,
            created_at: *dead_task_run.completed_at(),
            event_type: IncidentEventType::TaskRunIsDead,
            event_payload: None,
        };

        self.incident_event_repository
            .create_incident_event(transaction, task_run_started_event)
            .await?;
        self.incident_event_repository
            .create_incident_event(transaction, task_run_last_heartbeat_event)
            .await?;
        self.incident_event_repository
            .create_incident_event(transaction, task_run_dead_event)
            .await?;

        save_task_aggregate(
            &self.task_repository,
            &self.task_run_repository,
            transaction,
            TaskAggregate::Failing(dead_task_aggregate),
        )
        .await
        .context("Failed to save task aggregate")?;

        Ok(())
    }
}
