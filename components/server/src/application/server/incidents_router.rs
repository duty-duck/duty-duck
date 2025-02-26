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
        use_cases::incidents::{
            self, AcknowledgeIncidentError, CommentIncidentError, CommentIncidentRequest,
            GetIncidentError, GetIncidentResponse, GetIncidentTimelineError,
            GetIncidentTimelineParams, GetIncidentTimelineResponse, ListIncidentsError,
            ListIncidentsParams, ListIncidentsResponse, ResolveIncidentError,
        },
    },
};

pub fn incidents_router() -> Router<ApplicationState> {
    Router::new()
        .route("/", get(list_incidents_handler))
        .route(
            "/filterable-metadata",
            get(get_filterable_incident_metadata_handler),
        )
        .nest(
            "/{incident_id}",
            Router::new()
                .route("/", get(get_incident_handler))
                .route("/acknowledge", post(acknowledge_incident_handler))
                .route("/events", get(get_incident_timeline_handler))
                .route("/comment", post(comment_incident_handler))
                .route("/resolve", post(resolve_incident_handler)),
        )
}

/// List incidents
///
/// Returns a list of incidents matching the given filters.
/// If no filters are provided, all incidents are returned.
#[utoipa::path(
    get,
    path = "/incidents",
    responses(
        (status = 200, description = "Incidents fetched successfully", body = ListIncidentsResponse),
        (status = 403, description = "User is not authorized to fetch incidents"),
        (status = 500, description = "Technical failure occured while fetching incidents from the database")
    ),
    params(
        ListIncidentsParams
    )
)]
async fn list_incidents_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Query(params): Query<ListIncidentsParams>,
) -> impl IntoResponse {
    match incidents::list_incidents(
        &auth_context,
        &app_state.adapters.incident_repository,
        &app_state.adapters.user_repository,
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

/// Get a single incident by id
#[utoipa::path(
    get,
    path = "/incidents/{incident_id}",
    responses(
        (status = 200, description = "Incident fetched successfully", body = GetIncidentResponse),
        (status = 404, description = "Incident not found"),
        (status = 403, description = "User is not authorized to fetch incident"),
        (status = 500, description = "Technical failure occured while fetching incident from the database")
    ),
)]
async fn get_incident_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(incident_id): Path<Uuid>,
) -> impl IntoResponse {
    match incidents::get_incident(
        &auth_context,
        &app_state.adapters.incident_repository,
        &app_state.adapters.user_repository,
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

/// Get the timeline of events for an incident
#[utoipa::path(
    get,
    path = "/incidents/{incident_id}/events",
    responses(
        (status = 200, description = "Incident events fetched successfully", body = GetIncidentTimelineResponse),
        (status = 403, description = "User is not authorized to fetch incident events"),
        (status = 500, description = "Technical failure occured while fetching incident events from the database")
    ),
    params(
        GetIncidentTimelineParams
    )
)]
async fn get_incident_timeline_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(incident_id): Path<Uuid>,
    Query(params): Query<GetIncidentTimelineParams>,
) -> impl IntoResponse {
    match incidents::get_incident_timeline(
        &auth_context,
        &app_state.adapters.incident_event_repository,
        &app_state.adapters.user_repository,
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

/// Comment an incident
#[utoipa::path(
    post,
    path = "/incidents/{incident_id}/comment",
    responses(
        (status = 200, description = "Comment posted successfully"),
        (status = 404, description = "Incident not found"),
        (status = 403, description = "User is not authorized to comment incidents"),
        (status = 500, description = "Technical failure occured while saving the comment")
    ),
    request_body(
        content = CommentIncidentRequest,
        description = "The command to comment an incident",
        content_type = "application/json"
    ),
)]
async fn comment_incident_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(incident_id): Path<Uuid>,
    Json(request): Json<CommentIncidentRequest>,
) -> impl IntoResponse {
    match incidents::comment_incident(
        &auth_context,
        &app_state.adapters.incident_repository,
        &app_state.adapters.incident_event_repository,
        incident_id,
        request,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(CommentIncidentError::IncidentNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(CommentIncidentError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(CommentIncidentError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while commenting incident");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Acknowledge an incident
///
/// Acknowledging an incident lets anyone know you are working on a resolution
#[utoipa::path(
    post,
    path = "/incidents/{incident_id}/acknowledge",
    responses(
        (status = 200, description = "Incident acknowledged succesfully"),
        (status = 404, description = "Incident not found"),
        (status = 403, description = "User is not authorized to acknowledge incidents"),
        (status = 500, description = "Technical failure occured while saving the incident")
    ),
)]
async fn acknowledge_incident_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(incident_id): Path<Uuid>,
) -> impl IntoResponse {
    match incidents::acknowledge_incident(
        &auth_context,
        &app_state.adapters.incident_repository,
        &app_state.adapters.incident_event_repository,
        &app_state.adapters.incident_notification_repository,
        incident_id,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(AcknowledgeIncidentError::IncidentNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(AcknowledgeIncidentError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(AcknowledgeIncidentError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while acknowledging incident");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Resolve an incident manually
///
/// Marks an incident as resolved and cancels future notifications.
/// While you can manually resolve any incident, some incidents are automatically resolved when the root problem is resolved.
/// For example, when a monitor becomes helthy again, any related incident is resolved.
#[utoipa::path(
    post,
    path = "/incidents/{incident_id}/resolve",
    responses(
        (status = 200, description = "Incident resolved succesfully"),
        (status = 404, description = "Incident not found"),
        (status = 403, description = "User is not authorized to resolve incidents"),
        (status = 500, description = "Technical failure occured while saving the incident")
    ),
)]
async fn resolve_incident_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(incident_id): Path<Uuid>,
) -> impl IntoResponse {
    match incidents::resolve_incident_manually(
        &app_state.adapters.incident_repository,
        &app_state.adapters.incident_event_repository,
        &app_state.adapters.incident_notification_repository,
        &auth_context,
        incident_id,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(ResolveIncidentError::IncidentNotFound) => {
            (StatusCode::NOT_FOUND, "Incident not found").into_response()
        }
        Err(ResolveIncidentError::IncidentAlreadyResolved) => {
            (StatusCode::CONFLICT, "Incident already resolved").into_response()
        }
        Err(ResolveIncidentError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ResolveIncidentError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while acknowledging incident");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_filterable_incident_metadata_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
) -> impl IntoResponse {
    match incidents::get_filterable_incident_metadata(
        &auth_context,
        &app_state.adapters.incident_repository,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(e) => {
            warn!(error = ?e, "Technical failure occured while getting filterable incident metadata");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
