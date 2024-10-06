use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use axum_extra::extract::Query;
use tracing::warn;
use uuid::Uuid;

use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{
        entities::authorization::AuthContext,
        use_cases::incidents::{self, GetIncidentError, GetIncidentTimelineError, GetIncidentTimelineParams, ListIncidentsError, ListIncidentsParams},
    },
};

pub fn incidents_router() -> Router<ApplicationState> {
    Router::new()
        .route("/", get(list_incidents_handler))
        .route("/:incident_id", get(get_incident_handler))
        .route("/:incident_id/events", get(get_incident_timeline_handler))
}

async fn list_incidents_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Query(params): Query<ListIncidentsParams>,
) -> impl IntoResponse {
    match incidents::list_incidents(
        &auth_context,
        &app_state.adapters.incident_repository,
        params,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(ListIncidentsError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ListIncidentsError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting incidents from the database");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_incident_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(incident_id): Path<Uuid>,
) -> impl IntoResponse {
    match incidents::get_incident(
        &auth_context,
        &app_state.adapters.incident_repository,
        incident_id,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(GetIncidentError::IncidentNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(GetIncidentError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(GetIncidentError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting incident from the database");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_incident_timeline_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(incident_id): Path<Uuid>,
    Query(params): Query<GetIncidentTimelineParams>,
) -> impl IntoResponse {
    match incidents::get_incident_timeline(
        &auth_context,
        &app_state.adapters.incident_event_repository,
        incident_id,
        params,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(GetIncidentTimelineError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(GetIncidentTimelineError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting incidents timeline from the database");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
