use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::Query;
use tracing::warn;
use uuid::Uuid;

use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{
        entities::authorization::AuthContext,
        use_cases::{
            http_monitors::{
                self, CreateHttpMonitorCommand, CreateHttpMonitorError, ListHttpMonitorsError,
                ListHttpMonitorsParams, ReadHttpMonitorError, ToggleMonitorError,
                UpdateHttpMonitorCommand, UpdateHttpMonitorError,
            },
            incidents::{ListIncidentsError, ListIncidentsParams},
        },
    },
};

pub fn http_monitors_router() -> Router<ApplicationState> {
    Router::new()
        .route(
            "/",
            get(list_http_monitors_handler).post(create_http_monitor_handler),
        )
        .route(
            "/:monitor_id",
            get(get_http_monitor_handler).patch(update_http_monitor_handler),
        )
        .route(
            "/:monitor_id/incidents",
            get(get_http_monitor_incidents_handler),
        )
        .route("/:monitor_id/toggle", post(toggle_http_monitor_handler))
}

async fn get_http_monitor_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(monitor_id): Path<Uuid>,
) -> impl IntoResponse {
    match http_monitors::read_http_monitor(
        &auth_context,
        &app_state.adapters.http_monitors_repository,
        &app_state.adapters.incident_repository,
        monitor_id,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(ReadHttpMonitorError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(ReadHttpMonitorError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ReadHttpMonitorError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting a single HTTP monitor from the database");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_http_monitor_incidents_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(monitor_id): Path<Uuid>,
    Query(params): Query<ListIncidentsParams>,
) -> impl IntoResponse {
    match http_monitors::list_http_monitor_incidents(
        &auth_context,
        &app_state.adapters.incident_repository,
        monitor_id,
        params,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(ListIncidentsError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ListIncidentsError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting HTTP monitor incidents");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn toggle_http_monitor_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(monitor_id): Path<Uuid>,
) -> impl IntoResponse {
    match http_monitors::toggle_http_monitor(
        &auth_context,
        &app_state.adapters.http_monitors_repository,
        &app_state.adapters.incident_repository,
        monitor_id,
    )
    .await
    {
        Ok(_) => Json("Done").into_response(),
        Err(ToggleMonitorError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(ToggleMonitorError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ToggleMonitorError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while toggling HTTP monitor");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn list_http_monitors_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Query(params): Query<ListHttpMonitorsParams>,
) -> impl IntoResponse {
    match http_monitors::list_http_monitors(
        &auth_context,
        &app_state.adapters.http_monitors_repository,
        params,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(ListHttpMonitorsError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ListHttpMonitorsError::TechnicalFailure(e)) => {
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
        Err(CreateHttpMonitorError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting creating a new monitor");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn update_http_monitor_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(monitor_id): Path<Uuid>,
    Json(command): Json<UpdateHttpMonitorCommand>,
) -> impl IntoResponse {
    match http_monitors::update_http_monitor(
        &auth_context,
        &app_state.adapters.http_monitors_repository,
        monitor_id,
        command,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(UpdateHttpMonitorError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(UpdateHttpMonitorError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(UpdateHttpMonitorError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting creating a new monitor");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
