use axum::{
    extract::{Query, State}, http::StatusCode, response::IntoResponse, routing::get, Json, Router
};
use tracing::warn;

use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{
        entities::authorization::AuthContext,
        use_cases::http_monitors_crud_use_cases::{self, ListHttpMonitorsError},
    },
};

use super::paginations_params::PaginationParams;

pub fn http_monitors_router() -> Router<ApplicationState> {
    Router::new().route("/", get(list_http_monitors_handlers))
}

async fn list_http_monitors_handlers(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    match http_monitors_crud_use_cases::list_http_monitors(
        &auth_context,
        &app_state.adapters.http_monitors_repository,
        pagination.page_number(),
        pagination.items_per_page(),
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
