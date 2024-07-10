use std::sync::Arc;

use axum::extract::State;

use crate::infrastructure::{adapters::{
    http_monitor_repository_adapter::HttpMonitorRepositoryAdapter,
    organization_repository_adapter::OrganizationRepositoryAdapter,
    user_repository_adapter::UserRepositoryAdapter,
}, keycloak_client::KeycloakClient};

pub type ExtractAppState = State<ApplicationState>;

#[derive(Clone)]
pub struct ApplicationState {
    pub access_token_audience: String,
    pub adapters: Adapters,
    pub keycloak_client: Arc<KeycloakClient>
}

#[derive(Clone)]
pub struct Adapters {
    pub user_repository: UserRepositoryAdapter,
    pub organization_repository: OrganizationRepositoryAdapter,
    pub http_monitors_repository: HttpMonitorRepositoryAdapter,
}
