use anyhow::Context;
use chrono::Utc;
use thiserror::Error;

#[cfg(test)]
mod tests;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        incident::{IncidentPriority, IncidentSource, IncidentStatus},
        task::{get_task_aggregate, save_task_aggregate, TaskAggregate, TaskId},
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

#[derive(Debug, Error)]
pub enum ArchiveTaskError {
    #[error("Task not found")]
    TaskNotFound,
    #[error("Task cannot be archived while it is running")]
    CannotArchiveRunningTask,
    #[error("The task is already archived, nothing to do")]
    AlreadyArchived,
    #[error("User is not allowed to archive this task")]
    Forbidden,
    #[error("Technical error")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn archive_task<TR, TRR, IR, IER, INR>(
    task_repository: &TR,
    task_run_repository: &TRR,
    incident_repository: &IR,
    incident_event_repository: &IER,
    incident_notification_repository: &INR,
    auth_context: &AuthContext,
    task_id: TaskId,
) -> Result<(), ArchiveTaskError>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
    IR: IncidentRepository<Transaction = TR::Transaction>,
    IER: IncidentEventRepository<Transaction = TR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = TR::Transaction>,
{
    // Check permission
    if !auth_context.can(Permission::WriteTasks) {
        return Err(ArchiveTaskError::Forbidden);
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

    let archived_aggregate = match aggregate {
        None => return Err(ArchiveTaskError::TaskNotFound),
        Some(TaskAggregate::Archived(_)) => return Err(ArchiveTaskError::AlreadyArchived),
        Some(TaskAggregate::Running(_)) => return Err(ArchiveTaskError::CannotArchiveRunningTask),
        Some(TaskAggregate::Absent(agg)) => agg.archive(now),
        Some(TaskAggregate::Due(agg)) => agg.archive(now),
        Some(TaskAggregate::Late(agg)) => agg.archive(now),
        Some(TaskAggregate::Healthy(agg)) => agg.archive(now),
        Some(TaskAggregate::Failing(agg)) => agg.archive(now),
    };

    let task_id = *archived_aggregate.task().base().id();

    save_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        TaskAggregate::Archived(archived_aggregate),
    )
    .await
    .context("Failed to save archived task")?;

    // Retrieve all ongoing incidents for this task
    let ongoing_incidents = incident_repository
        .list_incidents(
            &mut tx,
            auth_context.active_organization_id,
            ListIncidentsOpts {
                include_statuses: &[IncidentStatus::Ongoing, IncidentStatus::ToBeConfirmed],
                include_priorities: &IncidentPriority::ALL,
                include_sources: &[IncidentSource::Task { id: task_id }],
                limit: 1,
                ..Default::default()
            },
        )
        .await?
        .incidents;

    for incident in ongoing_incidents {
        resolve_incident(
            &mut tx,
            incident_repository,
            incident_event_repository,
            incident_notification_repository,
            &incident,
            Some(auth_context.active_user_id),
        )
        .await
        .context("Failed to resolve incident")?;
    }

    task_repository
        .commit_transaction(tx)
        .await
        .context("Failed to commit transaction while archiving a task")?;

    Ok(())
}
