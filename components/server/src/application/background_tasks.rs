use std::sync::Arc;

use chrono::Utc;
use clap::*;

use crate::domain::use_cases::{
    http_monitors::ExecuteHttpMonitorsUseCase,
    incidents::ExecuteIncidentNotificationsUseCase,
    tasks::{
        CollectAbsentTasksUseCase, CollectDeadTaskRunsUseCase, CollectDueTasksUseCase,
        CollectLateTasksUseCase,
    },
};

use super::application_config::AppConfig;

#[derive(Subcommand, Debug)]
pub enum BackgroundTask {
    /// Execute due HTTP monitors
    HttpMonitors,
    /// Send pending incident notifications
    IncidentNotifications,
    /// Collect dead task runs
    CollectDeadTaskRuns,
    /// Collect due tasks
    CollectDueTasks,
    /// Collect late tasks
    CollectLateTasks,
    /// Collect absent tasks
    CollectAbsentTasks,
    /// Create monthly partitions for every partitioned table
    CreateMonthlyPartitions,
}

pub async fn run_background_task(task: BackgroundTask) -> anyhow::Result<()> {
    let config = Arc::new(AppConfig::load()?);
    let application_state = super::build_app_state(Arc::clone(&config)).await?;

    match task {
        BackgroundTask::CreateMonthlyPartitions => {
            application_state
                .adapters
                .task_run_repository
                .create_task_run_partition_for_month()
                .await?;
            application_state
                .adapters
                .incident_event_repository
                .create_incident_timeline_partition_for_month()
                .await?;
        }
        BackgroundTask::HttpMonitors => {
            ExecuteHttpMonitorsUseCase {
                http_monitor_repository: application_state
                    .adapters
                    .http_monitors_repository
                    .clone(),
                incident_repository: application_state.adapters.incident_repository.clone(),
                incident_event_repository: application_state
                    .adapters
                    .incident_event_repository
                    .clone(),
                incident_notification_repository: application_state
                    .adapters
                    .incident_notification_repository
                    .clone(),
                http_client: application_state.adapters.http_client.clone(),
                file_storage: application_state.adapters.file_storage.clone(),
            }
            .fetch_and_execute_due_http_monitors(
                0,
                config.http_monitors_executor.http_monitors_select_limit,
                config.http_monitors_executor.http_monitors_ping_concurrency,
            )
            .await?;
        }
        BackgroundTask::IncidentNotifications => {
            ExecuteIncidentNotificationsUseCase {
                organization_repository: application_state.adapters.organization_repository.clone(),
                incident_notification_repository: application_state
                    .adapters
                    .incident_notification_repository
                    .clone(),
                incident_event_repository: application_state
                    .adapters
                    .incident_event_repository
                    .clone(),
                push_notificaton_server: application_state
                    .adapters
                    .push_notification_server
                    .clone(),
                sms_notificaton_server: application_state.adapters.sms_notification_server.clone(),
                mailer: application_state.adapters.mailer.clone(),
                user_devices_repository: application_state.adapters.user_devices_repository.clone(),
                select_limit: config
                    .notifications_executor
                    .notifications_tasks_select_limit,
            }
            .fetch_and_execute_due_notifications()
            .await?;
        }
        BackgroundTask::CollectDeadTaskRuns => {
            CollectDeadTaskRunsUseCase {
                task_repository: application_state.adapters.task_repository.clone(),
                task_run_repository: application_state.adapters.task_run_repository.clone(),
                select_limit: config.dead_task_runs_collector.select_limit,
            }
            .collect_dead_task_runs()
            .await?;
        }
        BackgroundTask::CollectDueTasks => {
            CollectDueTasksUseCase {
                task_repository: application_state.adapters.task_repository.clone(),
                task_run_repository: application_state.adapters.task_run_repository.clone(),
                select_limit: config.due_tasks_collector.select_limit,
            }
            .collect_due_tasks()
            .await?;
        }
        BackgroundTask::CollectLateTasks => {
            CollectLateTasksUseCase {
                task_repository: application_state.adapters.task_repository.clone(),
                task_run_repository: application_state.adapters.task_run_repository.clone(),
                incident_repository: application_state.adapters.incident_repository.clone(),
                incident_event_repository: application_state
                    .adapters
                    .incident_event_repository
                    .clone(),
                incident_notification_repository: application_state
                    .adapters
                    .incident_notification_repository
                    .clone(),
                select_limit: config.late_tasks_collector.select_limit,
            }
            .collect_late_tasks(Utc::now())
            .await?;
        }
        BackgroundTask::CollectAbsentTasks => {
            CollectAbsentTasksUseCase {
                task_repository: application_state.adapters.task_repository.clone(),
                task_run_repository: application_state.adapters.task_run_repository.clone(),
                select_limit: config.absent_tasks_collector.select_limit,
            }
            .collect_absent_tasks()
            .await?;
        }
    }

    Ok(())
}
