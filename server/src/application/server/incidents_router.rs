use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use axum_extra::extract::Query;
use tracing::warn;

use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{
        entities::authorization::AuthContext,
        use_cases::incidents::{self, ListIncidentsError, ListIncidentsParams},
    },
};

pub fn incidents_router() -> Router<ApplicationState> {
    Router::new().route("/", get(list_incidents_handler))
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
        Err(ListIncidentsError::TechnicalError(e)) => {
            warn!(error = ?e, "Technical failure occured while getting incidents from the database");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
