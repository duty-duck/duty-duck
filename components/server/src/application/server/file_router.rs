use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{
        entities::authorization::AuthContext,
        use_cases::file_storage::{serve_file, ServeFileUseCaseError},
    },
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use uuid::Uuid;

pub(crate) fn file_router() -> Router<ApplicationState> {
    Router::new().route("/:file_id", get(serve_file_handler))
}

async fn serve_file_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    Path(file_id): Path<Uuid>,
) -> impl IntoResponse {
    match serve_file(&auth_context, &app_state.adapters.file_storage, file_id).await {
        Err(ServeFileUseCaseError::TechnicalFailure(_)) => {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Ok(url) => url.to_string().into_response(),
    }
}
