use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tracing::warn;

use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{
        entities::authorization::AuthContext,
        use_cases::{self, users::*},
    },
};

use super::user_devices_router::user_devices_router;

pub fn users_router() -> Router<ApplicationState> {
    Router::new()
        .route("/debug-auth-context", get(debug_auth_context_handler))
        .route("/signup", post(signup_handler))
        .route("/check-password", post(check_password_handler))
        .nest(
            "/me",
            Router::new()
                .nest("/devices", user_devices_router())
                .route("/", get(get_profile_handler).put(update_profile_handler)),
        )
}

async fn debug_auth_context_handler(auth_context: AuthContext) -> impl IntoResponse {
    Json(auth_context)
}

async fn get_profile_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
) -> impl IntoResponse {
    use use_cases::users::*;
    match get_user_profile(&auth_context, &app_state.adapters.user_repository).await {
        Ok(user) => Json(user).into_response(),
        Err(GetProfileError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(GetProfileError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Internal server error while getting a user's profile");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn update_profile_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Json(command): Json<UpdateProfileCommand>,
) -> impl IntoResponse {
    use use_cases::users::*;
    match update_user_profile(&auth_context, &app_state.adapters.user_repository, command).await {
        Ok(response) => Json(response).into_response(),
        Err(e @ UpdateProfileError::InvalidEmail) => {
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
        Err(e @ UpdateProfileError::PasswordTooWeak) => {
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
        Err(UpdateProfileError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Internal server error while updating a user's profile");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn check_password_handler(
    Json(body): Json<CheckPasswordStrengthCommand>,
) -> impl IntoResponse {
    Json(use_cases::users::check_password_strength(body))
}

async fn signup_handler(
    State(app_state): ExtractAppState,
    Json(command): Json<SignUpCommand>,
) -> impl IntoResponse {
    use use_cases::users::*;
    match sign_up(
        &app_state.adapters.organization_repository,
        &app_state.adapters.user_repository,
        command,
    )
    .await
    {
        Err(e @ SignUpError::InvalidEmail) => (StatusCode::BAD_REQUEST, e.to_string()),
        Err(e @ SignUpError::PasswordTooWeak) => (StatusCode::BAD_REQUEST, e.to_string()),
        Err(e @ SignUpError::UserAlreadyExists) => (StatusCode::CONFLICT, e.to_string()),
        Err(SignUpError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Internal server error while signing up a new user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            )
        }
        Ok(_) => (StatusCode::OK, "OK".to_string()),
    }
}
