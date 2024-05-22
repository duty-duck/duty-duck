use std::sync::Arc;

use axum::extract::State;
use rusty_paseto::core::{Local, PasetoSymmetricKey, V4};
use sea_orm::DatabaseConnection;

use crate::{mailer::Mailer, services::auth::AuthService};

pub struct AppEnv {
    pub auth_service: AuthService,
}

pub type ExtractAppEnv = State<Arc<AppEnv>>;

impl AppEnv {
    pub fn new(
        db: DatabaseConnection,
        mailer: Mailer,
        paseto_key: PasetoSymmetricKey<V4, Local>,
    ) -> Self {
        Self {
            auth_service: AuthService::new(db.clone(), mailer.clone(), paseto_key),
        }
    }
}
