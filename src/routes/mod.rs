pub mod assets;
mod auth;

use axum::{
    http::{header::CACHE_CONTROL, HeaderValue},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::{app_env::AppEnv, session::CurrentUser, views};

use self::{assets::assets_handler, auth::auth_router};

async fn root(user_opt: Option<CurrentUser>) -> impl IntoResponse {
    views::public::IndexTemplate {
        current_user: user_opt.map(|CurrentUser(user)| user),
    }
}

async fn pricing(user_opt: Option<CurrentUser>) -> impl IntoResponse {
    views::public::PricingTemplate {
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
            HeaderValue::from_static("max-age=600"),
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
