use askama_axum::*;
use axum::{routing::get, Router};
use std::sync::Arc;

use crate::state::AppState;

#[derive(Template)]
#[template(path = "auth/login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "auth/signup.html")]
struct SignupTemplate;

async fn login() -> impl IntoResponse {
    LoginTemplate
}

async fn signup() -> impl IntoResponse {
    SignupTemplate
}

pub fn auth_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", get(login))
        .route("/signup", get(signup))
}
