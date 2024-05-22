mod app_env;
mod mailer;
mod routes;
mod services;

use std::{env, sync::Arc};

use axum::Router;
use dotenv::dotenv;
use rusty_paseto::core::{Key, Local, PasetoSymmetricKey, V4};
use sea_orm::Database;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::EnvFilter;

use crate::{app_env::AppEnv, mailer::Mailer};

#[tokio::main]
async fn main() {
    // Read environment
    dotenv().unwrap();

    // initialize tracing
    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    // Initialize db
    let db =
        Database::connect(env::var("DATABASE_URL").expect("Missing env variable DATABASE_URL"))
            .await
            .expect("Failed to connect to the database");

    // Initialize mailer
    let mailer = Mailer::new().expect("Failed to initialize mailer");

    // Load paseto key
    let paseto_key = load_paseto_key();

    let app = Router::new()
        .nest("/", routes::public_site_router())
        .with_state(Arc::new(AppEnv::new(db, mailer, paseto_key)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server is listening on port 3000 (http://localhost:3000)");
    axum::serve(listener, app).await.unwrap();
}

fn load_paseto_key() -> PasetoSymmetricKey<V4, Local> {
    match env::var("PASETO_SECRET_KEY").ok() {
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
    }
}
