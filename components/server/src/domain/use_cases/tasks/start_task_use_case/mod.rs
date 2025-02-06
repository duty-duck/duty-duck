use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        incident::{IncidentPriority, IncidentSource, IncidentStatus},
        incident_event::{IncidentEvent, IncidentEventType},
        task::{
            get_task_aggregate, save_task_aggregate, HealthyTaskAggregate, RunningTaskAggregate,
            TaskAggregate, TaskId,
        },
    },
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::{IncidentRepository, ListIncidentsOpts},
        task_repository::TaskRepository,
        task_run_repository::TaskRunRepository,
    },
    use_cases::incidents::resolve_incident,
};

#[cfg(test)]
mod tests;

use super::CreateTaskCommand;

/// A command to start a task
#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StartTaskCommand {
    /// The properties of the new task to create if the task does not exist yet
    #[serde(default)]
    pub new_task: Option<NewTask>,
    /// Whether to abort the previous running task
    #[serde(default)]
    pub abort_previous_running_task: bool,
}

/// An optional command that can be used to create a task on-the-fly when starting a task run
#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NewTask {
    /// A human-readable name for the task
    pub name: Option<String>,
    /// A description of the task
    pub description: Option<String>,
    /// A cron schedule for the task, to make it a scheduled task
    pub cron_schedule: Option<String>,
    /// A number of seconds to wait, once this scheduled task is due, before the task is considered late
    pub start_window_seconds: Option<u32>,
    /// A number of seconds to wait, once this scheduled task is late, before the task is considered late
    pub lateness_window_seconds: Option<u32>,
    /// A number of seconds to wait, after the last heartbeat was received, if any, before the task is considered failed
    pub heartbeat_timeout_seconds: Option<u32>,
    /// The timezone to use for the task's schedule (defaults to UTC)
    pub schedule_timezone: Option<String>,
    /// Whether to send an email notification when an incident occurs for this task
    pub email_notification_enabled: Option<bool>,
    /// Whether to send a push notification when an incident occurs for this task
    pub push_notification_enabled: Option<bool>,
    /// Whether to send a SMS notification when an incident occurs for this task
    pub sms_notification_enabled: Option<bool>,
}

#[derive(Error, Debug)]
pub enum StartTaskError {
    #[error("Task not found")]
    TaskNotFound,
    #[error("Task already started")]
    TaskAlreadyStarted,
    #[error("User is not allowed to start this task")]
    Forbidden,
    #[error("Technical error")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub struct StartTaskUseCaseOpts<'a, TR, TRR, IR, IER, INR> {
    pub task_repository: &'a TR,
    pub task_run_repository: &'a TRR,
    pub incident_repository: &'a IR,
    pub incident_event_repository: &'a IER,
    pub incident_notification_repository: &'a INR,
    pub task_id: TaskId,
    pub command: Option<StartTaskCommand>,
}

#[tracing::instrument(skip(opts))]
pub async fn start_task_use_case<TR, TRR, IR, IER, INR>(
    auth_context: &AuthContext,
    opts: StartTaskUseCaseOpts<'_, TR, TRR, IR, IER, INR>,
) -> Result<(), StartTaskError>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
    IR: IncidentRepository<Transaction = TR::Transaction>,
    IER: IncidentEventRepository<Transaction = TR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = TR::Transaction>,
{
    let StartTaskUseCaseOpts {
        task_repository,
        task_run_repository,
        incident_repository,
        incident_event_repository,
        incident_notification_repository,
        task_id,
        command,
    } = opts;

    if !auth_context.can(Permission::WriteTaskRuns) {
        return Err(StartTaskError::Forbidden);
    }

    let mut tx = task_repository.begin_transaction().await?;
    let aggregate = get_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        auth_context.active_organization_id,
        &task_id,
    )
    .await
    .context("failed to get task aggregate from the database")?;

    let now = Utc::now();

    let (running_aggregate, was_late_or_absent): (RunningTaskAggregate, bool) = match aggregate {
        None => {
            let command = command.ok_or(StartTaskError::TaskNotFound)?;
            let new_task = command.new_task.ok_or(StartTaskError::TaskNotFound)?;
            let new_task = CreateTaskCommand {
                id: task_id,
                name: new_task.name,
                description: new_task.description,
                cron_schedule: new_task.cron_schedule,
                start_window_seconds: new_task.start_window_seconds,
                lateness_window_seconds: new_task.lateness_window_seconds,
                heartbeat_timeout_seconds: new_task.heartbeat_timeout_seconds,
                schedule_timezone: new_task.schedule_timezone,
                email_notification_enabled: new_task.email_notification_enabled,
                push_notification_enabled: new_task.push_notification_enabled,
                sms_notification_enabled: new_task.sms_notification_enabled,
                metadata: None,
            };
            let new_task = HealthyTaskAggregate::new(auth_context.active_organization_id, new_task)
                .context("failed to create a new task")?;
            let (running_aggregate, _) = new_task.start(now).context("failed to start new task")?;
            (running_aggregate, false)
        }
        Some(TaskAggregate::Running(t)) => {
            if command.is_some_and(|c| c.abort_previous_running_task) {
                let aborted_task = t
                    .mark_aborted(now)
                    .context("failed to abort running task")?;
                save_task_aggregate(
                    task_repository,
                    task_run_repository,
                    &mut tx,
                    TaskAggregate::Healthy(aborted_task.clone()),
                )
                .await
                .context("failed to save aborted task to the database")?;

                let (running_aggregate, _) = aborted_task
                    .start(now)
                    .context("failed to start aborted task")?;

                (running_aggregate, false)
            } else {
                return Err(StartTaskError::TaskAlreadyStarted);
            }
        }
        Some(TaskAggregate::Due(t)) => (t.start(now).context("failed to start due task")?, false),
        Some(TaskAggregate::Failing(t)) => {
            let (running_aggregate, _) = t.start(now).context("failed to start failing task")?;
            (running_aggregate, false)
        }
        Some(TaskAggregate::Healthy(t)) => {
            let (running_aggregate, _) = t.start(now).context("failed to start healthy task")?;
            (running_aggregate, false)
        }

        // if the task was previously late or absent, we need to check if there is an ongoing incident for this task and possibly resolve it
        // the `was_late_or_absent` flag captures that fact so we don't make unnecessary calls to the incident repository
        Some(TaskAggregate::Late(t)) => (t.start(now).context("failed to start late task")?, true),
        Some(TaskAggregate::Absent(t)) => {
            (t.start(now).context("failed to start absent task")?, true)
        }
    };

    // If the task was previously late or absent, we need to check if there is an ongoing incident for this task
    if was_late_or_absent {
        resolve_lateness_or_absence(
            incident_repository,
            incident_event_repository,
            incident_notification_repository,
            auth_context.active_organization_id,
            *running_aggregate.task().base().id(),
            &mut tx,
            now,
        )
        .await
        .context("failed to resolve lateness or absence incident")?;
    }

    save_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        TaskAggregate::Running(running_aggregate),
    )
    .await
    .context("failed to save task aggregate to the database")?;

    task_repository
        .commit_transaction(tx)
        .await
        .context("failed to commit transaction")?;

    Ok(())
}

/// Checks if there is an ongoing incident for the given task and resolves it
/// This function is called when the just-started task was previously late or absent
#[tracing::instrument(skip(
    incident_repository,
    incident_event_repository,
    incident_notification_repository,
    transaction
))]
async fn resolve_lateness_or_absence<IR, IER, INR>(
    incident_repository: &IR,
    incident_event_repository: &IER,
    incident_notification_repository: &INR,
    organization_id: Uuid,
    task_id: Uuid,
    transaction: &mut IR::Transaction,
    now: DateTime<Utc>,
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    let incident = match incident_repository
        .list_incidents(
            transaction,
            organization_id,
            ListIncidentsOpts {
                include_statuses: &[IncidentStatus::Ongoing],
                include_priorities: &IncidentPriority::ALL,
                include_sources: &[IncidentSource::Task { id: task_id }],
                limit: 1,
                ..Default::default()
            },
        )
        .await?
        .incidents
        .into_iter()
        .next()
    {
        Some(incident) => incident,
        None => return Ok(()),
    };

    // Create an event to indicate that the task has been started
    let event = IncidentEvent {
        organization_id,
        incident_id: incident.id,
        user_id: None,
        created_at: now,
        event_type: IncidentEventType::TaskSwitchedToRunning,
        event_payload: None,
    };

    incident_event_repository
        .create_incident_event(transaction, event)
        .await?;

    // Resolve the incident
    // (re-using an existing function which also takes care of cancelling notifications and creating a resolution event)
    resolve_incident(
        transaction,
        incident_repository,
        incident_event_repository,
        incident_notification_repository,
        &incident,
    )
    .await?;

    Ok(())
}
