pub mod assets;
mod auth;
mod dashboard;
mod public_routes;

use axum::{
    http::{header::CACHE_CONTROL, HeaderValue},
    response::IntoResponse,
    routing::get,
    Router,
};
use dashboard::dashboard_router;
use public_routes::public_site_router;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::{app_env::AppEnv, http_utils::session::CurrentUser, views};

use self::{assets::assets_handler, auth::auth_router};

pub fn all() -> Router<AppEnv> {
    Router::new()
        .nest("/auth", auth_router())
        .nest("/dashboard", dashboard_router())
        .nest("/", public_site_router())
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
