use crate::domain::{
    entities::{
        entity_metadata::MetadataFilter,
        incident::{IncidentCause, IncidentPriority, IncidentSource, IncidentStatus},
        incident_event::{IncidentEvent, IncidentEventType},
        task::{from_boundary, save_task_aggregate, LateTaskAggregate, TaskAggregate},
    },
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::{IncidentRepository, ListIncidentsOpts},
        task_repository::TaskRepository,
        task_run_repository::TaskRunRepository,
    },
    use_cases::{incidents::OrderIncidentsBy, shared::OrderDirection},
};

#[cfg(test)]
mod tests;

use anyhow::Context;
use chrono::{DateTime, Utc};
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::{error, info};

use super::{create_incident_for_late_aggregate, CreateIncidentForLateTaskOpts};

#[derive(Clone)]
pub struct CollectAbsentTasksUseCase<TR, TRR, IR, IER, INR> {
    pub task_repository: TR,
    pub task_run_repository: TRR,
    pub incident_repository: IR,
    pub incident_event_repository: IER,
    pub incident_notification_repository: INR,
    pub select_limit: u32,
}

impl<TR, TRR, IR, IER, INR> CollectAbsentTasksUseCase<TR, TRR, IR, IER, INR>
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
                            let now = Utc::now();
                            match executor.collect_absent_tasks(now).await {
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

    pub async fn collect_absent_tasks(&self, now: DateTime<Utc>) -> anyhow::Result<usize> {
        let mut transaction = self.task_repository.begin_transaction().await?;

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
            let late_aggregate = match aggregate {
                TaskAggregate::Late(agg) => agg,
                _ => anyhow::bail!("unexpected task aggregate type. Expected aggregate to be late. This is likely a bug in the SQL query used to retrieve aggregates"),
            };

            self.process_late_aggregate(&mut transaction, late_aggregate, now)
                .await?;
        }

        self.task_repository
            .commit_transaction(transaction)
            .await
            .context("Failed to commit transaction")?;

        Ok(task_aggregates_len)
    }

    #[tracing::instrument(skip(self, transaction))]
    async fn process_late_aggregate(
        &self,
        transaction: &mut TR::Transaction,
        late_aggregate: LateTaskAggregate,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let organization_id = *late_aggregate.task_base().organization_id();
        let task_id = *late_aggregate.task_base().id();
        let incident_sources = [IncidentSource::Task { id: task_id }];

        let ongoing_related_incident_from_db = self
            .incident_repository
            .list_incidents(
                transaction,
                organization_id,
                ListIncidentsOpts {
                    include_statuses: &[IncidentStatus::Ongoing],
                    include_priorities: &IncidentPriority::ALL,
                    include_sources: &incident_sources,
                    metadata_filter: MetadataFilter::default(),
                    limit: 1,
                    offset: 0,
                    from_date: None,
                    to_date: None,
                    order_by: OrderIncidentsBy::CreatedAt,
                    order_direction: OrderDirection::Desc,
                },
            )
            .await?
            .incidents
            .into_iter()
            .next();

            let task_was_due_at = late_aggregate.task().next_due_at();
            let task_ran_late_at = task_was_due_at
                + *late_aggregate.task_base().start_window()
                + *late_aggregate.task_base().lateness_window();

        // retrieve the the related incident or create it if it doesn't exist
        // absent tasks will usually have an existing incident, created when the task switched to late; however, incidents
        // can be manually resolved by the user, so we could need to create a new one
        let mut related_incident = match ongoing_related_incident_from_db {
            Some(incident) => incident,
            None => {
                let id = create_incident_for_late_aggregate(CreateIncidentForLateTaskOpts {
                    incident_repository: &self.incident_repository,
                    incident_event_repository: &self.incident_event_repository,
                    incident_notification_repository: &self.incident_notification_repository,
                    transaction,
                    aggregate: &late_aggregate,
                    task_ran_late_at,
                })
                .await?;
                self.incident_repository
                    .get_incident(transaction, organization_id, id)
                    .await?
                    .context("Failed to get incident after creating it")?
            }
        };

        // Update the incident cause
        if let Some(IncidentCause::ScheduledTaskIncidentCause(cause)) = &mut related_incident.cause {
            cause.task_switched_to_absent_at = Some(now);
            cause.task_was_due_at = task_was_due_at;
            cause.task_ran_late_at = Some(task_ran_late_at);
        };

        // Create additional event
        let task_switched_to_due_event = IncidentEvent {
            organization_id,
            incident_id: related_incident.id,
            user_id: None,
            created_at: now,
            event_type: IncidentEventType::TaskSwitchedToAbsent,
            event_payload: None,
        };

        // save the event
        self.incident_event_repository
            .create_incident_event(transaction, task_switched_to_due_event)
            .await?;

        // save the incident
        self.incident_repository.update_incident(transaction, related_incident).await?;

        // mark the task as absent and save it
        let absent_aggregate = late_aggregate.mark_absent(now).context("Failed to mark task aggregate as absent. This is likely a bug in the SQL query used to retrieve aggregates")?;

        save_task_aggregate(
            &self.task_repository,
            &self.task_run_repository,
            transaction,
            TaskAggregate::Absent(absent_aggregate),
        )
        .await
        .context("Failed to save task aggregate")?;

        Ok(())
    }
}
