mod assets;
mod state;

use std::sync::Arc;

use axum::{http::StatusCode, response::Html, routing::get, Router};
use state::ExtractState;
use tera::Context;
use tracing::info;

use crate::{assets::assets_handler, state::AppState};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/assets/*file", get(assets_handler))
        .with_state(Arc::new(AppState::new()));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server is listening on port 3000 (http://localhost:3000)");
    axum::serve(listener, app).await.unwrap();
}

async fn root(state: ExtractState) -> Result<Html<String>, StatusCode> {
    let page = state
        .templates
        .render("index.tera", &Context::new())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(page))
}
