use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tracing::warn;
use zxcvbn::zxcvbn;

use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::use_cases::sign_up_use_case::{self, *},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckPasswordStrength {
    password: String,
    first_name: String,
    last_name: String,
}

pub fn users_router() -> Router<ApplicationState> {
    Router::new()
        .route("/signup", post(signup_handler))
        .route("/check-password", post(check_password_handler))
}

async fn check_password_handler(Json(body): Json<CheckPasswordStrength>) -> impl IntoResponse {
    Json(zxcvbn(&body.password, &[&body.first_name, &body.last_name]))
}

async fn signup_handler(
    State(app_state): ExtractAppState,
    Json(command): Json<SignUpCommand>,
) -> impl IntoResponse {
    match sign_up_use_case::sign_up(
        &app_state.adapters.organization_repository,
        &app_state.adapters.user_repository,
        command,
    )
    .await
    {
        Err(sign_up_use_case::SignUpError::InvalidEmail) => {
            (StatusCode::BAD_REQUEST, "Invalid e-mail address")
        }
        Err(sign_up_use_case::SignUpError::PasswordTooWeak) => {
            (StatusCode::BAD_REQUEST, "Password is too weak")
        }
        Err(sign_up_use_case::SignUpError::UserAlreadyExists) => {
            (StatusCode::CONFLICT, "User already exists")
        }
        Err(sign_up_use_case::SignUpError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Internal server error while signing up a new user");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        }
        Ok(_) => (StatusCode::OK, "OK"),
    }
}
