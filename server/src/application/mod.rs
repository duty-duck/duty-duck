use std::sync::Arc;

use anyhow::Context;
use application_config::AppConfig;
use application_state::{Adapters, ApplicationState};
use reqwest::Url;
use sqlx::postgres::PgPoolOptions;

use crate::infrastructure::{
    adapters::{
        organization_repository_adapter::OrganizationRepositoryAdapter,
        user_repository_adapter::UserRepositoryAdapter,
    },
    keycloak_client::KeycloakClient,
};

pub mod application_config;
pub mod application_state;
pub mod built_info;
pub mod server;

pub async fn start_application() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    let application_state = build_app_state(&config).await?;

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
    };
    Ok(ApplicationState { adapters })
}
