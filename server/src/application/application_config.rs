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
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(AppConfig::init_from_env()?)
    }
}
