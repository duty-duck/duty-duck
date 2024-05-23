use std::{env, sync::Arc};

use axum::extract::State;
use rusty_paseto::core::{Key, Local, PasetoSymmetricKey, V4};
use sea_orm::DatabaseConnection;
use tracing::{info, warn};
use url::Url;

use crate::{mailer::Mailer, services::auth::AuthService};

pub struct AppConfig {
    pub public_url: Url,
    pub smtp_server_host: String,
    pub smtp_server_port: u16,
    pub smtp_disable_tls: bool,
    pub paseto_key: PasetoSymmetricKey<V4, Local>,
    pub database_url: String,
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
        let paseto_key = match env::var("PASETO_SECRET_KEY").ok() {
            None => {
                info!("Env variable PASETO_SECRET_KEY is not set, using a randomly-generated key");
                let random_key = Key::try_new_random().unwrap();
                PasetoSymmetricKey::from(random_key)
            }
            Some(k) => {
                warn!("Paseto key laoded from PASETO_SECRET_KEY");
                let k = Key::try_from(k.as_str()).unwrap();
                PasetoSymmetricKey::from(k)
            }
        };
        let database_url = env::var("DATABASE_URL").expect("Missing env variable DATABASE_URL");

        Self {
            public_url,
            paseto_key,
            smtp_disable_tls,
            smtp_server_host,
            smtp_server_port,
            database_url,
        }
    }
}

pub struct AppEnv {
    pub auth_service: AuthService,
}

pub type ExtractAppEnv = State<Arc<AppEnv>>;

impl AppEnv {
    pub fn new(app_config: Arc<AppConfig>, db: DatabaseConnection, mailer: Mailer) -> Self {
        Self {
            auth_service: AuthService::new(app_config.clone(), db.clone(), mailer.clone()),
        }
    }
}
