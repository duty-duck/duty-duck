pub mod assets;
mod auth;

use askama_axum::*;
use axum::{
    extract::Request,
    http::{header::CACHE_CONTROL, HeaderValue},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    routing::{get, Route},
    Json, Router,
};
use entity::user_account;
use std::sync::Arc;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::filters;
use crate::{app_env::AppEnv, session::CurrentUser};

use self::{assets::assets_handler, auth::auth_router};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    current_user: Option<user_account::Model>,
}

#[derive(Template)]
#[template(path = "pricing.html")]
struct PricingTemplate {
    current_user: Option<user_account::Model>,
}

async fn root(user_opt: Option<CurrentUser>) -> impl IntoResponse {
    IndexTemplate {
        current_user: user_opt.map(|CurrentUser(user)| user),
    }
}

async fn pricing(user_opt: Option<CurrentUser>) -> impl IntoResponse {
    PricingTemplate {
        current_user: user_opt.map(|CurrentUser(user)| user),
    }
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
}

pub fn all() -> Router<Arc<AppEnv>> {
    Router::new().nest("/", public_site_router()).route(
        "/assets/*file",
        get(assets_handler)
            // Serve static assets with aggressive HTTP caching
            .route_layer(SetResponseHeaderLayer::if_not_present(
                CACHE_CONTROL,
                HeaderValue::from_static("max-age=172800"),
            )),
    )
}
