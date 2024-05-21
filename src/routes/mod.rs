pub mod assets;
mod filters;

use askama_axum::*;
use axum::{routing::get, Router};
use std::sync::Arc;

use crate::state::AppState;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "pricing.html")]
struct PricingTemplate;

#[derive(Template)]
#[template(path = "auth/login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "auth/signup.html")]
struct SignupTemplate;

async fn root() -> impl IntoResponse {
    IndexTemplate
}

async fn pricing() -> impl IntoResponse {
    PricingTemplate
}

async fn login() -> impl IntoResponse {
    LoginTemplate
}

async fn signup() -> impl IntoResponse {
    SignupTemplate
}

pub fn public_site_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(root))
        .route("/pricing", get(pricing))
        .route("/auth/login", get(login))
        .route("/auth/signup", get(signup))
}
