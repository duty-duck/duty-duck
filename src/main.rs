mod app_env;
mod crypto;
mod http_client;
mod mailer;
mod routes;
mod services;
mod views;
mod http_utils;

use std::sync::Arc;

use app_env::AppConfig;
use axum::Router;
use dotenv::dotenv;
use sea_orm::Database;
use tracing::{info, level_filters::LevelFilter};
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

    // Load config
    let config = Arc::new(AppConfig::load());

    // Initialize db
    let db = Database::connect(&config.database_url)
        .await
        .expect("Failed to connect to the database");

    // Initialize mailer
    let mailer = Mailer::new(&config).expect("Failed to initialize mailer");

    let app = Router::new()
        .nest("/", routes::all())
        .with_state(AppEnv::new(config, db, mailer));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server is listening on port 3000 (http://localhost:3000)");
    axum::serve(listener, app).await.unwrap();
}
