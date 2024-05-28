use std::{env, ops::Deref, sync::Arc};

use axum::extract::State;
use sea_orm::DatabaseConnection;
use url::Url;

use crate::{
    crypto::SymetricEncryptionKey,
    mailer::Mailer,
    services::{auth::AuthService, http_monitors::HttpMonitorsService},
};

pub struct AppConfig {
    pub public_url: Url,
    pub smtp_server_host: String,
    pub smtp_server_port: u16,
    pub smtp_disable_tls: bool,
    pub paseto_key: SymetricEncryptionKey,
    pub database_url: String,
    pub user_agent: String,
}

impl AppConfig {
    pub fn load() -> Self {
        let smtp_server_host = env::var("SMTP_SERVER_HOST").unwrap_or("localhost".to_string());
        let smtp_server_port = match env::var("SMTP_SERVER_PORT") {
            Ok(var) => var
                .parse::<u16>()
                .expect("Failed to parse SMTP_SERVER_PORT"),
            _ => 25,
        };
        let smtp_disable_tls = env::var("SMTP_SKIP_TLS") == Ok("true".to_string());
        let public_url = env::var("PUBLIC_URL")
            .expect("Mising PUBLIC_URL variable")
            .parse::<Url>()
            .expect("Failed to parse PUBLIC_URL");
        let paseto_key = env::var("PASETO_SECRET_KEY")
            .expect("Missing env variable PASETO_SECRET_KEY")
            .parse::<SymetricEncryptionKey>()
            .unwrap();

        let database_url = env::var("DATABASE_URL").expect("Missing env variable DATABASE_URL");
        let user_agent = env::var("USER_AGENT").unwrap_or_else(|_| {
            "Mozilla/5.0+(compatible; DutyDuck/2.0; http://ww.dutyduck.com/)".to_string()
        });

        Self {
            public_url,
            paseto_key,
            smtp_disable_tls,
            smtp_server_host,
            smtp_server_port,
            database_url,
            user_agent,
        }
    }
}

#[derive(Clone)]
pub struct AppEnv {
    inner: Arc<AppEnvInner>,
}

pub struct AppEnvInner {
    pub auth_service: AuthService,
    pub http_monitors_service: HttpMonitorsService,
    pub config: Arc<AppConfig>,
}

pub type ExtractAppEnv = State<AppEnv>;

impl AppEnv {
    pub fn new(app_config: Arc<AppConfig>, db: DatabaseConnection, mailer: Mailer) -> Self {
        let inner = AppEnvInner {
            config: app_config.clone(),
            auth_service: AuthService::new(app_config.clone(), db.clone(), mailer.clone()),
            http_monitors_service: HttpMonitorsService::new(app_config.clone(), db.clone()),
        };
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl Deref for AppEnv {
    type Target = AppEnvInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
