use std::sync::Arc;

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
            incident_repository_adapter::IncidentRepositoryAdapter,
            organization_repository_adapter::OrganizationRepositoryAdapter,
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
    let config = AppConfig::load()?;
    let application_state = build_app_state(&config).await?;
    let _http_monitors_tasks = use_cases::http_monitors::spawn_http_monitors_execution_tasks(
        config.http_monitors_concurrent_tasks,
        application_state.adapters.http_monitors_repository.clone(),
        application_state.adapters.incident_repository.clone(),
        application_state.adapters.http_client.clone(),
        config.http_monitors_select_size,
        config.http_monitors_ping_concurrency,
    );

    server::start_server(application_state, config.server_port).await?;
    Ok(())
}

async fn build_app_state(config: &AppConfig) -> anyhow::Result<ApplicationState> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
        .with_context(|| "Failed to connect to the database")?;

    let keycloak_url =
        Url::parse(&config.keycloak_url).with_context(|| "Failed to parse keycloak URL")?;

    let keycloak_client = Arc::new(
        KeycloakClient::new(
            keycloak_url,
            &config.keycloak_realm,
            &config.keycloak_client,
            &config.keycloak_secret,
        )
        .await?,
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
        user_devices_repository: UserDevicesRepositoryAdapter { pool: pool.clone() },
        http_client: HttpClientAdapter::new(config),
    };
    Ok(ApplicationState {
        adapters,
        keycloak_client: keycloak_client.clone(),
        access_token_audience: config.access_token_audience.clone(),
    })
}
