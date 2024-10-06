use std::sync::Arc;

use axum::extract::State;

use crate::infrastructure::{
    adapters::{
        http_client_adapter::HttpClientAdapter, http_monitor_repository_adapter::HttpMonitorRepositoryAdapter, incident_event_repository_adapter::IncidentEventRepositoryAdapter, incident_notification_repository_adapter::IncidentNotificationRepositoryAdapter, incident_repository_adapter::IncidentRepositoryAdapter, mailer_adapter::MailerAdapter, organization_repository_adapter::OrganizationRepositoryAdapter, push_notification_server_adapter::PushNotificationServerAdapter, user_devices_repository_adapter::UserDevicesRepositoryAdapter, user_repository_adapter::UserRepositoryAdapter
    },
    keycloak_client::KeycloakClient,
};

use super::application_config::AppConfig;

pub type ExtractAppState = State<ApplicationState>;

#[derive(Clone)]
pub struct ApplicationState {
    pub access_token_audience: String,
    pub adapters: Adapters,
    pub keycloak_client: Arc<KeycloakClient>,
    pub config: Arc<AppConfig>
}

#[derive(Clone)]
pub struct Adapters {
    pub user_repository: UserRepositoryAdapter,
    pub organization_repository: OrganizationRepositoryAdapter,
    pub http_monitors_repository: HttpMonitorRepositoryAdapter,
    pub incident_repository: IncidentRepositoryAdapter,
    pub incident_notification_repository: IncidentNotificationRepositoryAdapter,
    pub incident_event_repository: IncidentEventRepositoryAdapter,
    pub http_client: HttpClientAdapter,
    pub user_devices_repository: UserDevicesRepositoryAdapter,
    pub push_notification_server: PushNotificationServerAdapter,
    pub mailer: MailerAdapter,
}
