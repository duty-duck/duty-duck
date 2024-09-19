mod auth_context_extractor;
mod http_monitors_router;
mod incidents_router;
mod users_router;
mod user_devices_router;
mod organizations_router;

use std::time::Duration;

use axum::{routing::get, Json, Router};
use http_monitors_router::http_monitors_router;
use incidents_router::incidents_router;
use organizations_router::organizations_router;
use tokio::signal;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing::info;
use users_router::users_router;

use super::{application_state::ApplicationState, built_info::build_info_json};

pub async fn start_server(application_state: ApplicationState, port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .nest("/users", users_router())
        .nest("/http-monitors", http_monitors_router())
        .nest("/incidents", incidents_router())
        .nest("/organizations", organizations_router())
        .route("/", get(|| async { Json(build_info_json()) }))
        .layer(CorsLayer::permissive())
        .with_state(application_state)
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(30)),
        ));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;

    info!(port = port, "Application is listenning on port {port}!");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
