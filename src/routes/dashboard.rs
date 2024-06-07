use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use tracing::warn;
use url::Url;

use crate::{
    app_env::{AppEnv, ExtractAppEnv},
    http_utils::{form::SecureForm, session::*},
    services::http_monitors::{CreateMonitorParams, GetMonitorParams},
    views::{self, dashboard::monitors::CreateMonitorForm, Pagination},
};

async fn index(CurrentUser { user, .. }: CurrentUser) -> impl axum::response::IntoResponse {
    views::dashboard::DashboardHome { user }
}

async fn monitors_index(
    env: ExtractAppEnv,
    CurrentUser { user, tenant }: CurrentUser,
    pagination: Option<Query<Pagination>>,
) -> impl axum::response::IntoResponse {
    let monitors = env
        .http_monitors_service
        .list_monitors(
            tenant.id,
            GetMonitorParams {
                page: pagination.as_ref().map_or(0, |p| p.page),
                items_per_page: pagination.map_or(20, |p| p.per_page),
            },
        )
        .await
        // TODO: handle error here using Internal error page
        .unwrap();

    views::dashboard::monitors::MonitorsIndex { user, monitors }
}

async fn new_monitor(
    CurrentUser { user, .. }: CurrentUser,
    session: Session,
) -> impl axum::response::IntoResponse {
    views::dashboard::monitors::NewMonitorForm {
        user,
        csrf_token: session.csrf_token,
        form: CreateMonitorForm {
            url: String::new(),
            interval_seconds: 300,
        },
        error: None,
    }
}

async fn handle_new_monitor(
    env: ExtractAppEnv,
    CurrentUser { user, tenant }: CurrentUser,
    session: Session,
    form: SecureForm<CreateMonitorForm>,
) -> impl axum::response::IntoResponse {
    let url = match form.url.parse::<Url>() {
        Ok(url) => url,
        Err(_) => {
            return views::dashboard::monitors::NewMonitorForm {
                user,
                csrf_token: session.csrf_token,
                form: form.payload,
                error: Some("Invalid URL"),
            }
            .into_response()
        }
    };
    let params = CreateMonitorParams {
        url,
        interval_seconds: form.payload.interval_seconds,
    };

    match env
        .http_monitors_service
        .create_monitor(tenant.id, params)
        .await
    {
        Err(e) => {
            warn!(error = ?e, "Failed to create an HTTP monitor");
            views::dashboard::monitors::NewMonitorForm {
                user,
                csrf_token: session.csrf_token,
                form: form.payload,
                error: Some("A technical failure occured on our end, please try again."),
            }
            .into_response()
        }
        Ok(_) => Redirect::to("/dashboard/monitors").into_response(),
    }
}

pub fn dashboard_router() -> Router<AppEnv> {
    Router::new()
        .route("/", get(index))
        .route("/monitors", get(monitors_index).post(handle_new_monitor))
        .route("/monitors/new", get(new_monitor))
}
