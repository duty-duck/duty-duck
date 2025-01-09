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
        entities::{authorization::AuthContext, http_monitor::HttpMonitor},
        use_cases::{
            http_monitors::{
                self, ArchiveMonitorError, CreateHttpMonitorCommand, CreateHttpMonitorError, ListHttpMonitorsError, ListHttpMonitorsParams, ListHttpMonitorsResponse, ReadHttpMonitorError, ToggleMonitorError, UpdateHttpMonitorCommand, UpdateHttpMonitorError
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
            "/filterable-metadata",
            get(get_filterable_http_monitor_metadata_handler),
        )
        .route(
            "/{monitor_id}",
            get(get_http_monitor_handler).patch(update_http_monitor_handler),
        )
        .route(
            "/{monitor_id}/incidents",
            get(get_http_monitor_incidents_handler),
        )
        .route("/{monitor_id}/toggle", post(toggle_http_monitor_handler))
        .route("/{monitor_id}/archive", post(archive_http_monitor_handler))
}

/// Get a single HTTP monitor
///
/// Returns a single HTTP monitor by its ID.
#[utoipa::path(
    get,
    path = "/http-monitors/:monitor_id",
    responses(
        (status = 200, description = "HTTP monitor fetched successfully", body = HttpMonitor),
        (status = 403, description = "User is not authorized to fetch the HTTP monitor"),
        (status = 404, description = "HTTP monitor not found"),
        (status = 500, description = "Technical failure occured while fetching the HTTP monitor from the database")
    )
)]
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
        &app_state.adapters.user_repository,
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

/// Toggle a HTTP monitor
///
/// Toggles a HTTP monitor by its ID.
#[utoipa::path(
    post,
    path = "/http-monitors/:monitor_id/toggle",
    responses(
        (status = 200, description = "HTTP monitor toggled successfully"),
        (status = 403, description = "User is not authorized to toggle the HTTP monitor"),
        (status = 404, description = "HTTP monitor not found"),
        (status = 400, description = "HTTP monitor is archived and cannot be toggled"),
        (status = 500, description = "Technical failure occured while toggling the HTTP monitor")
    )
)]
async fn toggle_http_monitor_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(monitor_id): Path<Uuid>,
) -> impl IntoResponse {
    match http_monitors::toggle_http_monitor(
        &auth_context,
        &app_state.adapters.http_monitors_repository,
        &app_state.adapters.incident_repository,
        &app_state.adapters.incident_event_repository,
        &app_state.adapters.incident_notification_repository,
        monitor_id,
    )
    .await
    {
        Ok(_) => Json("Done").into_response(),
        Err(ToggleMonitorError::NotFound) => {
            (StatusCode::NOT_FOUND, "Monitor not found").into_response()
        }
        Err(ToggleMonitorError::MonitorIsArchived) => (
            StatusCode::BAD_REQUEST,
            "Monitor is archived and cannot be toggled",
        )
            .into_response(),
        Err(ToggleMonitorError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ToggleMonitorError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while toggling HTTP monitor");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Archive a HTTP monitor
///
/// Archives a HTTP monitor by its ID.
#[utoipa::path(
    post,
    path = "/http-monitors/:monitor_id/archive",
    responses(
        (status = 200, description = "HTTP monitor archived successfully"),
        (status = 403, description = "User is not authorized to archive the HTTP monitor"),
        (status = 404, description = "HTTP monitor not found"),
        (status = 400, description = "HTTP monitor is already archived"),
        (status = 500, description = "Technical failure occured while archiving the HTTP monitor")
    )
)]
async fn archive_http_monitor_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(monitor_id): Path<Uuid>,
) -> impl IntoResponse {
    match http_monitors::archive_http_monitor(
        &auth_context,
        &app_state.adapters.http_monitors_repository,
        &app_state.adapters.incident_repository,
        &app_state.adapters.incident_event_repository,
        &app_state.adapters.incident_notification_repository,
        monitor_id,
    )
    .await
    {
        Ok(_) => Json("Done").into_response(),
        Err(ArchiveMonitorError::NotFound) => {
            (StatusCode::NOT_FOUND, "Monitor not found").into_response()
        }
        Err(ArchiveMonitorError::MonitorIsArchived) => (
            StatusCode::BAD_REQUEST,
            "Monitor is archived and cannot be archived",
        )
            .into_response(),
        Err(ArchiveMonitorError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ArchiveMonitorError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while archiving HTTP monitor");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// List HTTP monitors
///
/// Returns a list of HTTP monitors matching the given filters.
#[utoipa::path(
    get,
    path = "/http-monitors",
    responses(
        (status = 200, description = "HTTP monitors fetched successfully", body = ListHttpMonitorsResponse),
        (status = 403, description = "User is not authorized to fetch HTTP monitors"),
        (status = 500, description = "Technical failure occured while fetching HTTP monitors from the database")
    ),
    params(
        ListHttpMonitorsParams
    )
)]
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

/// Create a new HTTP monitor
///
/// Creates a new HTTP monitor.
#[utoipa::path(
    post,
    path = "/http-monitors",
    responses(
        (status = 200, description = "HTTP monitor created successfully", body = HttpMonitor),
        (status = 403, description = "User is not authorized to create a HTTP monitor"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Technical failure occured while creating the HTTP monitor")
    )
)]
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
        Err(CreateHttpMonitorError::InvalidUrl(_)) => StatusCode::BAD_REQUEST.into_response(),
        Err(CreateHttpMonitorError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting creating a new monitor");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Update a HTTP monitor
///
/// Updates a HTTP monitor by its ID.
#[utoipa::path(
    patch,
    path = "/http-monitors/:monitor_id",
    responses(
        (status = 200, description = "HTTP monitor updated successfully", body = HttpMonitor),
        (status = 403, description = "User is not authorized to update the HTTP monitor"),
        (status = 404, description = "HTTP monitor not found"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Technical failure occured while updating the HTTP monitor")
    )
)]
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
        Err(UpdateHttpMonitorError::Forbidden) => {
            (StatusCode::FORBIDDEN, "Forbidden to update this monitor").into_response()
        }
        Err(UpdateHttpMonitorError::NotFound) => {
            (StatusCode::NOT_FOUND, "Monitor not found").into_response()
        }
        Err(UpdateHttpMonitorError::InvalidUrl(_)) => {
            (StatusCode::BAD_REQUEST, "Invalid URL").into_response()
        }
        Err(UpdateHttpMonitorError::InvalidRequestTimeout) => {
            (StatusCode::BAD_REQUEST, "Invalid request timeout").into_response()
        }
        Err(UpdateHttpMonitorError::MonitorIsArchived) => (
            StatusCode::BAD_REQUEST,
            "Monitor is archived and cannot be updated",
        )
            .into_response(),
        Err(UpdateHttpMonitorError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting creating a new monitor");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_filterable_http_monitor_metadata_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
) -> impl IntoResponse {
    match http_monitors::get_filterable_http_monitor_metadata(
        &auth_context,
        &app_state.adapters.http_monitors_repository,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(e) => {
            warn!(error = ?e, "Technical failure occured while getting filterable http monitor metadata");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
