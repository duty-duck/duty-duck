use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct KeycloakConfig {
    #[envconfig(from = "KEYCLOAK_PUBLIC_URL")]
    pub public_url: String,
    #[envconfig(from = "KEYCLOAK_PRIVATE_URL")]
    pub private_url: String,
    #[envconfig(from = "KEYCLOAK_REALM", default = "master")]
    pub realm: String,
    #[envconfig(from = "KEYCLOAK_CLIENT")]
    pub client_id: String,
    #[envconfig(from = "KEYCLOAK_SECRET")]
    pub client_secret: String,
    #[envconfig(from = "ACCESS_TOKEN_AUDIENCE", default = "dutyduck-dashboard")]
    pub access_token_audience: String,
}

#[derive(Envconfig)]
pub struct DbConfig {
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,
    #[envconfig(from = "DATABASE_MAX_CONNECTIONS", default = "10")]
    pub database_max_connections: u32,
}

#[derive(Envconfig)]
pub struct SmtpConfig {
    #[envconfig(from = "SMTP_SERVER_HOST")]
    pub server_host: String,
    #[envconfig(from = "SMTP_SERVER_PORT")]
    pub server_port: u16,
    #[envconfig(from = "SMTP_SERVER_DISABLE_TLS")]
    pub disable_tls: bool,
    #[envconfig(from = "SMTP_USERNAME")]
    pub username: Option<String>,
    #[envconfig(from = "SMTP_PASSWORD")]
    pub password: Option<String>,
}

#[derive(Envconfig)]
pub struct NotificationsExecutorConfig {
    #[envconfig(from = "NOTIFICATIONS_CONCURRENT_TASKS", default = "1")]
    pub notifications_concurrent_tasks: usize,
    #[envconfig(from = "NOTIFICATIONS_TASKS_INTERVAL", default = "1")]
    pub notifications_tasks_interval_seconds: u64,
    #[envconfig(from = "NOTIFICATIONS_TASKS_SELECT_LIMIT", default = "500")]
    pub notifications_tasks_select_limit: u32,
}

#[derive(Envconfig)]
pub struct HttpMonitorsExecutorConfig {
    #[envconfig(from = "HTTP_MONITORS_CONCURRENT_TASKS", default = "2")]
    pub http_monitors_concurrent_tasks: usize,
    #[envconfig(from = "HTTP_MONITORS_PING_CONCURRENCY", default = "100")]
    pub http_monitors_ping_concurrency: usize,
    #[envconfig(from = "HTTP_MONITORS_SELECT_LIMIT", default = "500")]
    pub http_monitors_select_limit: u32,
    #[envconfig(from = "HTTP_MONITORS_EXECUTOR_INTERVAL_SECONDS", default = "2")]
    pub http_monitors_executor_interval_seconds: u64,
    #[envconfig(from = "BROWSER_SERVICE_GRPC_ADDRESS")]
    pub browser_service_grpc_address: String,
}

#[derive(Envconfig)]
pub struct DeadTaskRunsCollectorConfig {
    #[envconfig(from = "DEAD_TASK_RUNS_COLLECTOR_INTERVAL", default = "10")]
    pub interval_seconds: u64,
    #[envconfig(from = "DEAD_TASK_RUNS_COLLECTOR_SELECT_LIMIT", default = "500")]
    pub select_limit: u32,
    #[envconfig(from = "DEAD_TASK_RUNS_COLLECTOR_CONCURRENT_TASKS", default = "1")]
    pub concurrent_tasks: usize,
}

#[derive(Envconfig)]
pub struct FileStorageConfig {
    #[envconfig(from = "FILE_STORAGE_BUCKET_NAME")]
    pub bucket_name: String,
}

#[derive(Envconfig)]
pub struct AppConfig {
    #[envconfig(from = "SERVER_PORT")]
    pub server_port: u16,

    #[envconfig(nested = true)]
    pub db: DbConfig,

    #[envconfig(from = "PUBLIC_URL")]
    pub public_url: String,

    #[envconfig(nested = true)]
    pub keycloak: KeycloakConfig,

    #[envconfig(nested = true)]
    pub file_storage: FileStorageConfig,

    #[envconfig(nested = true)]
    pub notifications_executor: NotificationsExecutorConfig,

    #[envconfig(nested = true)]
    pub http_monitors_executor: HttpMonitorsExecutorConfig,

    #[envconfig(nested = true)]
    pub dead_task_runs_collector: DeadTaskRunsCollectorConfig,

    #[envconfig(nested = true)]
    pub smtp: SmtpConfig
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(AppConfig::init_from_env()?)
    }
}
