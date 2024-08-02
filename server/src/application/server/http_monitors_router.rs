use axum_extra::extract::Query;
use axum::{
    extract::{State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tracing::warn;

use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{
        entities::authorization::AuthContext,
        use_cases::http_monitors::{
            self, CreateHttpMonitorCommand, CreateHttpMonitorError, ListHttpMonitorsError, ListHttpMonitorsParams,
        },
    },
};

pub fn http_monitors_router() -> Router<ApplicationState> {
    Router::new().route(
        "/",
        get(list_http_monitors_handler).post(create_http_monitor_handler),
    )
}

async fn list_http_monitors_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Query(params): Query<ListHttpMonitorsParams>,
) -> impl IntoResponse {
    match http_monitors::list_http_monitors(
        &auth_context,
        &app_state.adapters.http_monitors_repository,
        params
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(ListHttpMonitorsError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ListHttpMonitorsError::TechnicalError(e)) => {
            warn!(error = ?e, "Technical failure occured while getting http monitors from the database");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn create_http_monitor_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Json(command): Json<CreateHttpMonitorCommand>,
) -> impl IntoResponse {
    match http_monitors::create_http_monitor(
        &auth_context,
        &app_state.adapters.http_monitors_repository,
        command,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(CreateHttpMonitorError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(CreateHttpMonitorError::TechnicalError(e)) => {
            warn!(error = ?e, "Technical failure occured while getting creating a new monitor");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
