mod routes;
mod state;

use std::sync::Arc;

use axum::Router;
use tracing::info;

use crate::state::AppState;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .nest("/", routes::public_site_router())
        .with_state(Arc::new(AppState::new()));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server is listening on port 3000 (http://localhost:3000)");
    axum::serve(listener, app).await.unwrap();
}
