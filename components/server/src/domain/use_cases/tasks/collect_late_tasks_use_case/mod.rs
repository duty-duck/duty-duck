use crate::domain::{
    entities::{
        incident::*, incident_event::*, incident_notification::IncidentNotificationPayload, task::*,
    },
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::IncidentRepository, task_repository::TaskRepository,
        task_run_repository::TaskRunRepository,
    },
    use_cases::incidents::{create_incident, NotificationOpts},
};

#[cfg(test)]
mod tests;

use anyhow::Context;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct CollectLateTasksUseCase<TR, TRR, IR, IER, INR> {
    pub task_repository: TR,
    pub task_run_repository: TRR,
    pub incident_repository: IR,
    pub incident_event_repository: IER,
    pub incident_notification_repository: INR,
    pub select_limit: u32,
}

impl<TR, TRR, IR, IER, INR> CollectLateTasksUseCase<TR, TRR, IR, IER, INR>
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
            info!("No task will be spawned. You need to call the `run collect-late-tasks` command manually to collect late tasks");
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
                            match executor.collect_late_tasks(now).await {
                                Ok(late_tasks) if late_tasks > 0 => {
                                    info!(late_tasks, "Collected {} late tasks", late_tasks);
                                }
                                Err(e) => {
                                    error!(error = ?e, "Failed to collect late tasks")
                                }
                                Ok(_) => {}
                            }
                        }
                        _ = tokio::signal::ctrl_c() => {
                            info!("Shutting down late tasks collector task");
                            break;
                        }
                    }
                }
            });
        }

        join_set
    }

    pub async fn collect_late_tasks(&self, now: DateTime<Utc>) -> anyhow::Result<usize> {
        let mut transaction = self.task_repository.begin_transaction().await?;

        let task_aggregates: Vec<TaskAggregate> = self
            .task_repository
            .list_due_tasks_running_late(&mut transaction, now, self.select_limit)
            .await
            .context("Failed to get late tasks from the database")?
            .into_iter()
            .map(|task| from_boundary(task, None))
            .collect::<anyhow::Result<Vec<_>>>()
            .context("Failed to convert late tasks from boundaries to task aggregates")?;

        let task_aggregates_len = task_aggregates.len();

        // turn every task aggregate into a failing one and save it
        for aggregate in task_aggregates {
            let late_aggregate = match aggregate {
                TaskAggregate::Due(agg) => {
                    agg.mark_late(now).context("Failed to mark task aggregate as late. This is likely a bug in the SQL query used to retrieve aggregates")?
                },
                _ => anyhow::bail!("unexpected task aggregate type. This is likely a bug in the SQL query used to retrieve aggregates"),
            };

            // Create an incident for the late task
            self.create_incident_for_late_aggregate(&mut transaction, &late_aggregate, now)
                .await?;

            // Save the late task aggregate
            save_task_aggregate(
                &self.task_repository,
                &self.task_run_repository,
                &mut transaction,
                TaskAggregate::Late(late_aggregate),
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

    async fn create_incident_for_late_aggregate(
        &self,
        transaction: &mut TR::Transaction,
        aggregate: &LateTaskAggregate,
        task_ran_late_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        create_incident_for_late_aggregate(CreateIncidentForLateTaskOpts {
            incident_repository: &self.incident_repository,
            incident_event_repository: &self.incident_event_repository,
            incident_notification_repository: &self.incident_notification_repository,
            transaction,
            aggregate,
            task_ran_late_at,
        })
        .await?;

        Ok(())
    }
}

pub struct CreateIncidentForLateTaskOpts<'a, TX, IR, IER, INR> {
    pub incident_repository: &'a IR,
    pub incident_event_repository: &'a IER,
    pub incident_notification_repository: &'a INR,
    pub transaction: &'a mut TX,
    pub aggregate: &'a LateTaskAggregate,
    pub task_ran_late_at: DateTime<Utc>,
}

/// Creates a lateness incident for a late task
///
/// This function:
///   - Creates an incident
///   - Creates a creation event for the incident
///   - Creates notifications for the incident
///   - Creates an incident event for the task switching to late
///   - Creates an incident event for the task switching to due
///
/// This function is used to create the incident from the [collect late tasks use case](CollectLateTasksUseCase),
/// and also, from other use cases that may need to create a lateness incident, which is why this function is public.
pub async fn create_incident_for_late_aggregate<IR, IER, INR>(
    opts: CreateIncidentForLateTaskOpts<'_, IR::Transaction, IR, IER, INR>,
) -> anyhow::Result<Uuid>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    let CreateIncidentForLateTaskOpts {
        incident_repository,
        incident_event_repository,
        incident_notification_repository,
        transaction,
        aggregate,
        task_ran_late_at,
    } = opts;

    let task_was_due_at = aggregate.task().next_due_at();

    debug!(
        task_id = ?aggregate.user_id(),
        "Task is running late, creating an incident"
    );

    let task_base = aggregate.task_base();

    let cause = IncidentCause::ScheduledTaskIncidentCause(ScheduledTaskIncidentCause {
        task_id: *task_base.id(),
        task_user_id: task_base.user_id().clone(),
        task_was_due_at,
        task_ran_late_at: Some(task_ran_late_at),
        task_switched_to_absent_at: None,
    });

    let new_incident = NewIncident {
        organization_id: *task_base.organization_id(),
        created_by: None,
        status: IncidentStatus::Ongoing,
        priority: IncidentPriority::Major,
        source: IncidentSource::Task {
            id: *task_base.id(),
        },
        cause: Some(cause.clone()),
        metadata: task_base.metadata().clone(),
    };

    // Create the incident, the creation event and the notifications
    let incident_id = create_incident(
        transaction,
        incident_repository,
        incident_event_repository,
        incident_notification_repository,
        task_ran_late_at,
        new_incident,
        // TODO: let users configure this
        Some(NotificationOpts {
            send_sms: false,
            send_push_notification: false,
            send_email: false,
            notification_payload: IncidentNotificationPayload {
                incident_cause: cause,
                incident_task_id: Some(task_base.user_id().clone()),
                incident_http_monitor_url: None,
            },
        }),
    )
    .await?;

    // Create additional events for the incident
    let task_switched_to_due_event = IncidentEvent {
        organization_id: *task_base.organization_id(),
        incident_id,
        user_id: None,
        created_at: task_was_due_at,
        event_type: IncidentEventType::TaskSwitchedToDue,
        event_payload: None,
    };
    let task_switched_to_late_event = IncidentEvent {
        organization_id: *task_base.organization_id(),
        incident_id,
        user_id: None,
        created_at: task_ran_late_at,
        event_type: IncidentEventType::TaskSwitchedToLate,
        event_payload: None,
    };

    incident_event_repository
        .create_incident_event(transaction, task_switched_to_due_event)
        .await?;
    incident_event_repository
        .create_incident_event(transaction, task_switched_to_late_event)
        .await?;

    Ok(incident_id)
}
