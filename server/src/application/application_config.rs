use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct AppConfig {
    #[envconfig(from = "SERVER_PORT")]
    pub server_port: u16,
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,
    #[envconfig(from = "DATABASE_MAX_CONNECTIONS", default = "10")]
    pub database_max_connections: u32,
    #[envconfig(from = "PUBLIC_URL")]
    pub public_url: String,
    #[envconfig(from = "KEYCLOAK_URL")]
    pub keycloak_url: String,
    #[envconfig(from = "KEYCLOAK_REALM", default = "master")]
    pub keycloak_realm: String,
    #[envconfig(from = "KEYCLOAK_CLIENT")]
    pub keycloak_client: String,
    #[envconfig(from = "KEYCLOAK_SECRET")]
    pub keycloak_secret: String,
    #[envconfig(from = "ACCESS_TOKEN_AUDIENCE", default = "dutyduck-dashboard")]
    pub access_token_audience: String,
    #[envconfig(from = "HTTP_MONITORS_CONCURRENT_TASKS", default = "2")]
    pub http_monitors_concurrent_tasks: usize,
    #[envconfig(from = "HTTP_MONITORS_PING_CONCURRENCY", default = "100")]
    pub http_monitors_ping_concurrency: usize,
    #[envconfig(from = "HTTP_MONITORS_SELECT_SIZE", default = "500")]
    pub http_monitors_select_size: u32,
    #[envconfig(
        from = "USER_AGENT",
        default = "Mozilla/5.0+(compatible; DutyDuck/2.0; http://ww.dutyduck.com/)"
    )]
    pub user_agent: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(AppConfig::init_from_env()?)
    }
}
