use axum::{routing::get, Router};

use crate::{app_env::AppEnv, session::CurrentUser, views};

async fn index(CurrentUser(user): CurrentUser) -> impl axum::response::IntoResponse {
    views::dashboard::DashboardHome {
        user
    }
}

async fn monitors(CurrentUser(user): CurrentUser) -> impl axum::response::IntoResponse {
    views::dashboard::MonitorsIndex {
        user
    }
}

pub fn dashboard_router() -> Router<AppEnv> {
    Router::new()
        .route("/", get(index))
        .route("/monitors", get(monitors))
}