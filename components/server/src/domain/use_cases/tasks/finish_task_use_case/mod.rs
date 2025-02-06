use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission}, incident::{
            IncidentCause, IncidentPriority, IncidentSource, IncidentStatus, NewIncident,
            TaskRunIncidentCause,
        }, incident_event::{IncidentEvent, IncidentEventType}, incident_notification::IncidentNotificationPayload, task::{
            get_task_aggregate, save_task_aggregate, FailingTaskAggregate, FailingTaskRun,
            TaskAggregate, TaskId,
        }, task_run::TaskRunStatus
    },
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::IncidentRepository, task_repository::TaskRepository,
        task_run_repository::TaskRunRepository,
    },
    use_cases::incidents::{create_incident, NotificationOpts},
};

#[derive(Error, Debug)]
pub enum FinishTaskError {
    #[error("User is not allowed to finish this task")]
    Forbidden,
    #[error("Task not found")]
    NotFound,
    #[error("Task is not running")]
    TaskIsNotRunning,
    #[error("Technical failure occured while finishing a task")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum FinishedTaskStatus {
    Success,
    Failure,
    Aborted,
}

#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FinishTaskCommand {
    pub status: FinishedTaskStatus,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub error_message: Option<String>,
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(
    auth_context,
    task_repository,
    task_run_repository,
    incident_repository,
    incident_event_repository,
    incident_notification_repository
))]
pub async fn finish_task_use_case<TR, TRR, IR, IER, INR>(
    auth_context: &AuthContext,
    task_repository: &TR,
    task_run_repository: &TRR,
    incident_repository: &IR,
    incident_event_repository: &IER,
    incident_notification_repository: &INR,
    task_id: TaskId,
    command: FinishTaskCommand,
) -> Result<(), FinishTaskError>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
    IR: IncidentRepository<Transaction = TR::Transaction>,
    IER: IncidentEventRepository<Transaction = TR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = TR::Transaction>,
{
    if !auth_context.can(Permission::WriteTaskRuns) {
        return Err(FinishTaskError::Forbidden);
    }

    let mut tx = task_repository.begin_transaction().await?;
    let aggregate = get_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        auth_context.active_organization_id,
        &task_id,
    )
    .await?;

    let now = Utc::now();
    let updated_aggregate = match aggregate {
        None => return Err(FinishTaskError::NotFound),
        Some(TaskAggregate::Running(t)) => match command.status {
            FinishedTaskStatus::Success => TaskAggregate::Healthy(
                t.mark_finished(now, command.exit_code)
                    .context("failed to finish running task")?,
            ),
            FinishedTaskStatus::Failure => {
                let failing_aggregate = t
                    .mark_failed(now, command.exit_code, command.error_message)
                    .context("failed to finish running task")?;
                process_failing_aggregate(
                    incident_repository,
                    incident_event_repository,
                    incident_notification_repository,
                    &mut tx,
                    &failing_aggregate,
                    now,
                )
                .await?;
                TaskAggregate::Failing(failing_aggregate)
            }
            FinishedTaskStatus::Aborted => TaskAggregate::Healthy(
                t.mark_aborted(now)
                    .context("failed to finish running task")?,
            ),
        },
        Some(_) => return Err(FinishTaskError::TaskIsNotRunning),
    };

    save_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        updated_aggregate,
    )
    .await?;
    task_repository.commit_transaction(tx).await?;

    Ok(())
}

/// A function called when a task is finished with a failure status
/// It will create an incident and create notifications
#[tracing::instrument(skip(
    incident_repository,
    incident_event_repository,
    incident_notification_repository,
    transaction
))]
async fn process_failing_aggregate<IR, IER, INR>(
    incident_repository: &IR,
    incident_event_repository: &IER,
    incident_notification_repository: &INR,
    transaction: &mut IR::Transaction,
    failing_aggregate: &FailingTaskAggregate,
    now: DateTime<Utc>,
) -> anyhow::Result<()>
where
    IR: IncidentRepository,
    IER: IncidentEventRepository<Transaction = IR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
{
    let organization_id = *failing_aggregate.task().base().organization_id();
    let task_id = *failing_aggregate.task().base().id();
    let task_user_id = failing_aggregate.task().base().user_id();

    let task_run = match failing_aggregate.task_run() {
        FailingTaskRun::Failed(f) => f,
        FailingTaskRun::Dead(_) => unreachable!(),
    };

    let incident_cause = IncidentCause::TaskRunIncidentCause(TaskRunIncidentCause {
        task_id: task_user_id.clone(),
        task_run_id: *task_run.id(),
        task_run_started_at: *task_run.started_at(),
        task_run_finished_at: Some(*task_run.completed_at()),
        task_run_status: TaskRunStatus::Failed,
    });
    let incident_id = create_incident(
        transaction,
        incident_repository,
        incident_event_repository,
        incident_notification_repository,
        now,
        NewIncident {
            organization_id,
            created_by: None,
            status: IncidentStatus::Ongoing,
            priority: IncidentPriority::Major,
            source: IncidentSource::Task { id: task_id },
            cause: Some(incident_cause.clone()),
            metadata: failing_aggregate.task().base().metadata().clone(),
        },
        Some(NotificationOpts {
            send_sms: *failing_aggregate.task().base().sms_notification_enabled(),
            send_push_notification: *failing_aggregate.task().base().push_notification_enabled(),
            send_email: *failing_aggregate.task().base().email_notification_enabled(),
            notification_payload: IncidentNotificationPayload {
                incident_cause,
                incident_http_monitor_url: None,
                incident_task_id: Some(task_user_id.clone()),
            },
        }),
    )
    .await
    .context("failed to create incident for failed task")?;

    // Create additional events
    let task_run_started_event = IncidentEvent {
        organization_id,
        incident_id,
        user_id: None,
        created_at: *task_run.started_at(),
        event_type: IncidentEventType::TaskRunStarted,
        event_payload: None,
    };

    let task_run_failed_event = IncidentEvent {
        organization_id,
        incident_id,
        user_id: None,
        created_at: now,
        event_type: IncidentEventType::TaskRunFailed,
        event_payload: None,
    };

    // save the events
    incident_event_repository
        .create_incident_event(transaction, task_run_started_event)
        .await?;
    incident_event_repository
        .create_incident_event(transaction, task_run_failed_event)
        .await?;

    Ok(())
}

