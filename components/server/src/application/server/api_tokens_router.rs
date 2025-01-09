use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{
        entities::authorization::AuthContext,
        use_cases::auth::{
            create_api_access_token, delete_api_access_token, list_api_access_tokens, CreateApiAccessTokenError, CreateApiTokenRequest, DeleteApiAccessTokenError, ListApiAccessTokensError
        },
    },
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, post},
    Json, Router,
};
use tracing::warn;
use uuid::Uuid;

pub fn api_tokens_router() -> Router<ApplicationState> {
    Router::new()
        .route("/", post(create_api_access_token_handler).get(list_api_access_tokens_handler))
        .route("/{api_token}", delete(delete_api_token_handler))
}

pub async fn create_api_access_token_handler(
    auth_context: AuthContext,
    State(application_state): ExtractAppState,
    Json(request): Json<CreateApiTokenRequest>,
) -> impl IntoResponse {
    match create_api_access_token(
        &auth_context,
        &application_state.adapters.api_token_repository,
        request,
    )
    .await
    {
        Ok(response) => Json(response).into_response(),
        Err(CreateApiAccessTokenError::InsufficientPermissions) => {
            StatusCode::FORBIDDEN.into_response()
        }
        Err(CreateApiAccessTokenError::InvalidExpirationDate) => {
            StatusCode::BAD_REQUEST.into_response()
        }
        Err(CreateApiAccessTokenError::TechnicalFailure(e)) => {
            warn!(
                error = ?e,
                "Technical failure occured while creating API token"
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn delete_api_token_handler(
    auth_context: AuthContext,
    State(application_state): ExtractAppState,
    Path(api_token): Path<Uuid>,
) -> impl IntoResponse {
    match delete_api_access_token(
        &auth_context,
        &application_state.adapters.api_token_repository,
        api_token,
    )
    .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(DeleteApiAccessTokenError::ApiTokenNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(DeleteApiAccessTokenError::TechnicalFailure(e)) => {
            warn!(
                error = ?e,
                "Technical failure occured while deleting API token"
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn list_api_access_tokens_handler(
    auth_context: AuthContext,
    State(application_state): ExtractAppState,
) -> impl IntoResponse {
    match list_api_access_tokens(
        &auth_context,
        &application_state.adapters.api_token_repository,
    )
    .await
    {
        Ok(response) => Json(response).into_response(),
        Err(ListApiAccessTokensError::TechnicalFailure(e)) => {
            warn!(
                error = ?e,
                "Technical failure occured while getting API tokens from the database"
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
