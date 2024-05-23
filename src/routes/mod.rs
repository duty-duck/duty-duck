pub mod assets;
mod auth;
mod filters;

use askama_axum::*;
use axum::{
    http::{header::CACHE_CONTROL, HeaderValue},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::app_env::AppEnv;

use self::{assets::assets_handler, auth::auth_router};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "pricing.html")]
struct PricingTemplate;

async fn root() -> impl IntoResponse {
    IndexTemplate
}

async fn pricing() -> impl IntoResponse {
    PricingTemplate
}

pub fn public_site_router() -> Router<Arc<AppEnv>> {
    Router::new()
        .route("/", get(root))
        .route("/pricing", get(pricing))
        .nest("/auth", auth_router())
        .layer(SetResponseHeaderLayer::if_not_present(
            CACHE_CONTROL,
            HeaderValue::from_static("max-age=1800"),
        ))
        .route(
            "/assets/*file",
            get(assets_handler)
                // Serve static assets with aggressive HTTP caching
                .route_layer(SetResponseHeaderLayer::if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static("max-age=172800"),
                )),
        )
}
