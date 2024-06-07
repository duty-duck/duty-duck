use axum::{
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};

use crate::{app_env::AppEnv, http_utils::tenant_based_routing::CurrentTenant, views};

async fn root(current_tenant: Option<CurrentTenant>) -> impl IntoResponse {
    if current_tenant.is_some() {
        return Redirect::to("/dashboard").into_response();
    }
    views::public::IndexTemplate {}.into_response()
}

async fn pricing(current_tenant: Option<CurrentTenant>) -> impl IntoResponse {
    if current_tenant.is_some() {
        return Redirect::to("/dashboard").into_response();
    }
    views::public::PricingTemplate {}.into_response()
}

pub fn public_site_router() -> Router<AppEnv> {
    Router::new()
        .route("/", get(root))
        .route("/pricing", get(pricing))
}
