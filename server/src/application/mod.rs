use std::{sync::Arc, time::Duration};

use anyhow::Context;
use application_config::AppConfig;
use application_state::{Adapters, ApplicationState};
use reqwest::Url;
use sqlx::postgres::PgPoolOptions;

use crate::{
    domain::use_cases,
    infrastructure::{
        adapters::{
            http_client_adapter::HttpClientAdapter,
            http_monitor_repository_adapter::HttpMonitorRepositoryAdapter,
            incident_notification_repository_adapter::IncidentNotificationRepositoryAdapter,
            incident_repository_adapter::IncidentRepositoryAdapter, mailer_adapter::{MailerAdapter, MailerAdapterConfig},
            organization_repository_adapter::OrganizationRepositoryAdapter,
            push_notification_server_adapter::PushNotificationServerAdapter,
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

    // TODO: implement graceful shutdown here
    let _http_monitors_tasks = use_cases::http_monitors::spawn_http_monitors_execution_tasks(
        config.http_monitors_concurrent_tasks,
        application_state.adapters.http_monitors_repository.clone(),
        application_state.adapters.incident_repository.clone(),
        application_state.adapters.http_client.clone(),
        config.http_monitors_select_size,
        config.http_monitors_ping_concurrency,
    );

    let _new_incident_notification_task =
        use_cases::incidents::spawn_new_incident_notification_tasks(
            config.notifications_concurrent_tasks,
            Duration::from_secs(config.notifications_tasks_interval_seconds),
            application_state.adapters.organization_repository.clone(),
            application_state
                .adapters
                .incident_notification_repository
                .clone(),
            application_state.adapters.push_notification_server.clone(),
            application_state.adapters.mailer.clone(),
            application_state.adapters.user_devices_repository.clone(),
            config.notifications_tasks_select_size,
        );

    tokio::select! {
        _ = _http_monitors_tasks.join_all() => (),
        _ = _new_incident_notification_task.join_all() => (),
        _ = server::start_server(application_state, config.server_port) => (),
    }

    Ok(())
}

async fn build_app_state(config: Arc<AppConfig>) -> anyhow::Result<ApplicationState> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
        .with_context(|| "Failed to connect to the database")?;

    let keycloak_client = Arc::new(
        KeycloakClient::new(
            Url::parse(&config.keycloak_public_url)
                .with_context(|| "Failed to parse keycloak public URL")?,
            Url::parse(&config.keycloak_private_url)
                .with_context(|| "Failed to parse keycloak private URL")?,
            &config.keycloak_realm,
            &config.keycloak_client,
            &config.keycloak_secret,
        )
        .await
        .with_context(|| "Failed to create Keycloak client")?,
    );

    let adapters = Adapters {
        organization_repository: OrganizationRepositoryAdapter {
            keycloak_client: keycloak_client.clone(),
        },
        user_repository: UserRepositoryAdapter {
            keycloak_client: keycloak_client.clone(),
        },
        http_monitors_repository: HttpMonitorRepositoryAdapter { pool: pool.clone() },
        incident_repository: IncidentRepositoryAdapter { pool: pool.clone() },
        incident_notification_repository: IncidentNotificationRepositoryAdapter {
            pool: pool.clone(),
        },
        user_devices_repository: UserDevicesRepositoryAdapter { pool: pool.clone() },
        http_client: HttpClientAdapter::new(&config),
        push_notification_server: PushNotificationServerAdapter::new().await?,
        mailer: MailerAdapter::new(MailerAdapterConfig {
            smtp_server_host: config.smtp_server_host.clone(),
            smtp_server_port: config.smtp_server_port,
            smtp_disable_tls: config.smtp_disable_tls,
            smtp_username: config.smtp_username.clone(),
            smtp_password: config.smtp_password.clone(),
        })?,
    };
    Ok(ApplicationState {
        config: config.clone(),
        adapters,
        keycloak_client: keycloak_client.clone(),
        access_token_audience: config.access_token_audience.clone(),
    })
}
