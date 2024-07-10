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
        use_cases::sign_up_use_case::{self, *},
    },
};

pub fn users_router() -> Router<ApplicationState> {
    Router::new()
        .route("/debug-auth-context", get(debug_auth_context_handler))
        .route("/signup", post(signup_handler))
        .route("/check-password", post(check_password_handler))
}

async fn debug_auth_context_handler(auth_context: AuthContext) -> impl IntoResponse {
    Json(auth_context)
}

async fn check_password_handler(
    Json(body): Json<CheckPasswordStrengthCommand>,
) -> impl IntoResponse {
    Json(sign_up_use_case::check_password_strength(body))
}

async fn signup_handler(
    State(app_state): ExtractAppState,
    Json(command): Json<SignUpCommand>,
) -> impl IntoResponse {
    use sign_up_use_case::*;
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
