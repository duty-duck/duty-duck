use std::{sync::Arc, time::Duration};

use anyhow::Context;
use application_config::AppConfig;
use application_state::{Adapters, ApplicationState};
use reqwest::Url;
use sqlx::postgres::PgPoolOptions;

use crate::{
    domain::use_cases::{
        http_monitors::ExecuteHttpMonitorsUseCase, incidents::ExecuteIncidentNotificationsUseCase,
        tasks::ClearDeadTaskRunsUseCase,
    },
    infrastructure::{
        adapters::{
            api_access_token_repository_adapter::ApiAccessTokenRepositoryAdapter,
            file_storage_adapter::FileStorageAdapter,
            http_client_adapter::HttpClientAdapter,
            http_monitor_repository_adapter::HttpMonitorRepositoryAdapter,
            incident_event_repository_adapter::IncidentEventRepositoryAdapter,
            incident_notification_repository_adapter::IncidentNotificationRepositoryAdapter,
            incident_repository_adapter::IncidentRepositoryAdapter,
            mailer_adapter::{MailerAdapter, MailerAdapterConfig},
            organization_repository_adapter::OrganizationRepositoryAdapter,
            push_notification_server_adapter::PushNotificationServerAdapter,
            sms_notification_server_adapter::SmsNotificationServerAdapter,
            task_repository_adapter::TaskRepositoryAdapter,
            task_run_repository_adapter::TaskRunRepositoryAdapter,
            user_devices_repository_adapter::UserDevicesRepositoryAdapter,
            user_repository_adapter::UserRepositoryAdapter,
        },
        keycloak_client::KeycloakClient,
    },
};

pub mod application_config;
pub mod application_state;
pub mod built_info;
pub mod server;

pub async fn start_application() -> anyhow::Result<()> {
    let config = Arc::new(AppConfig::load()?);
    let application_state = build_app_state(Arc::clone(&config)).await?;

    let http_monitors_use_case = ExecuteHttpMonitorsUseCase {
        http_monitor_repository: application_state.adapters.http_monitors_repository.clone(),
        incident_repository: application_state.adapters.incident_repository.clone(),
        incident_event_repository: application_state.adapters.incident_event_repository.clone(),
        incident_notification_repository: application_state
            .adapters
            .incident_notification_repository
            .clone(),
        http_client: application_state.adapters.http_client.clone(),
        file_storage: application_state.adapters.file_storage.clone(),
    };
    // TODO: implement graceful shutdown for these tasks
    let http_monitors_tasks = http_monitors_use_case.spawn_http_monitors_execution_tasks(
        config.http_monitors_executor.http_monitors_concurrent_tasks,
        config.http_monitors_executor.http_monitors_select_limit,
        config.http_monitors_executor.http_monitors_ping_concurrency,
        Duration::from_secs(
            config
                .http_monitors_executor
                .http_monitors_executor_interval_seconds,
        ),
    );

    let execute_incident_notifications = ExecuteIncidentNotificationsUseCase {
        organization_repository: application_state.adapters.organization_repository.clone(),
        incident_notification_repository: application_state
            .adapters
            .incident_notification_repository
            .clone(),
        incident_event_repository: application_state.adapters.incident_event_repository.clone(),
        push_notificaton_server: application_state.adapters.push_notification_server.clone(),
        sms_notificaton_server: application_state.adapters.sms_notification_server.clone(),
        mailer: application_state.adapters.mailer.clone(),
        user_devices_repository: application_state.adapters.user_devices_repository.clone(),
        select_limit: config
            .notifications_executor
            .notifications_tasks_select_limit,
    };
    let incident_notifications_tasks = execute_incident_notifications.spawn_tasks(
        config.notifications_executor.notifications_concurrent_tasks,
        Duration::from_secs(
            config
                .notifications_executor
                .notifications_tasks_interval_seconds,
        ),
    );

    let dead_task_runs_collector = ClearDeadTaskRunsUseCase {
        task_repository: application_state.adapters.task_repository.clone(),
        task_run_repository: application_state.adapters.task_run_repository.clone(),
        select_limit: config.dead_task_runs_collector.select_limit,
    };
    let dead_task_runs_collector_tasks = dead_task_runs_collector.spawn_tasks(
        config.dead_task_runs_collector.concurrent_tasks,
        Duration::from_secs(config.dead_task_runs_collector.interval_seconds),
    );

    let server_task = tokio::spawn(server::start_server(application_state, config.server_port));

    // Wait for all tasks to finish
    let _ = tokio::join!(
        http_monitors_tasks.join_all(),
        incident_notifications_tasks.join_all(),
        dead_task_runs_collector_tasks.join_all(),
        server_task
    );

    Ok(())
}

async fn build_app_state(config: Arc<AppConfig>) -> anyhow::Result<ApplicationState> {
    let pool = PgPoolOptions::new()
        .max_connections(config.db.database_max_connections)
        .connect(&config.db.database_url)
        .await
        .with_context(|| "Failed to connect to the database")?;

    let keycloak_client = Arc::new(
        KeycloakClient::new(
            Url::parse(&config.keycloak.public_url)
                .with_context(|| "Failed to parse keycloak public URL")?,
            Url::parse(&config.keycloak.private_url)
                .with_context(|| "Failed to parse keycloak private URL")?,
            &config.keycloak.realm,
            &config.keycloak.client_id,
            &config.keycloak.client_secret,
        )
        .await
        .with_context(|| "Failed to create Keycloak client")?,
    );

    let adapters = Adapters {
        organization_repository: OrganizationRepositoryAdapter {
            keycloak_client: keycloak_client.clone(),
        },
        user_repository: UserRepositoryAdapter::new(keycloak_client.clone()),
        api_token_repository: ApiAccessTokenRepositoryAdapter { pool: pool.clone() },
        http_monitors_repository: HttpMonitorRepositoryAdapter { pool: pool.clone() },
        incident_repository: IncidentRepositoryAdapter { pool: pool.clone() },
        incident_event_repository: IncidentEventRepositoryAdapter::new(pool.clone()),
        incident_notification_repository: IncidentNotificationRepositoryAdapter {
            pool: pool.clone(),
        },
        user_devices_repository: UserDevicesRepositoryAdapter { pool: pool.clone() },
        http_client: HttpClientAdapter::new(&config)
            .await
            .context("Failed to create http client adapter")?,
        push_notification_server: PushNotificationServerAdapter::new()
            .await
            .context("Failed to create push notification server adapter")?,
        mailer: MailerAdapter::new(MailerAdapterConfig {
            smtp_server_host: config.smtp.server_host.clone(),
            smtp_server_port: config.smtp.server_port,
            smtp_disable_tls: config.smtp.disable_tls,
            smtp_username: config.smtp.username.clone(),
            smtp_password: config.smtp.password.clone(),
        })
        .context("Failed to create mailer adapter")?,
        sms_notification_server: SmsNotificationServerAdapter::new()
            .await
            .context("Failed to create SMS notification server adapter")?,
        file_storage: FileStorageAdapter::new(config.file_storage.bucket_name.clone())
            .await
            .context("Failed to create file storage adapter")?,
        task_repository: TaskRepositoryAdapter { pool: pool.clone() },
        task_run_repository: TaskRunRepositoryAdapter::new(pool.clone()),
    };
    Ok(ApplicationState {
        config: config.clone(),
        adapters,
        keycloak_client: keycloak_client.clone(),
        access_token_audience: config.keycloak.access_token_audience.clone(),
    })
}
