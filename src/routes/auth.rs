use axum::extract::Path;
use axum::response::{AppendHeaders, IntoResponse, Redirect};
use axum::routing::post;
use axum::{routing::get, Form, Router};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tracing::error;
use uuid::Uuid;

use crate::app_env::{AppEnv, ExtractAppEnv};
use crate::services::auth::email_confirmation::{ConfirmEmailError, EmailConfirmationToken};
use crate::services::auth::{LoginError, SignUpError};
use crate::session::{Session, SetSession};
use crate::views;

async fn login() -> impl IntoResponse {
    views::auth::Login::default()
}

async fn signup() -> impl IntoResponse {
    views::auth::Signup::default()
}

async fn handle_login(
    env: ExtractAppEnv,
    Form(form): Form<views::auth::LogInForm>,
) -> impl IntoResponse {
    match env.auth_service.log_in(&form.email, &form.password).await {
        Ok(user) => {
            let session = Session { user_id: user.id };
            (
                SetSession(session, &env.config),
                Redirect::to("/dashboard"),
            )
                .into_response()
        }
        Err(e) => {
            // Delay the response for a few seconds to avoid malicious users from making too many attempts
            tokio::time::sleep(Duration::from_secs(2)).await;
            views::auth::HandleLogin {
                error: Some(e),
                ..Default::default()
            }
            .into_response()
        }
    }
}

async fn handle_signup(
    state: ExtractAppEnv,
    form: Form<views::auth::SignupForm>,
) -> impl IntoResponse {
    match form.0.validate() {
        Ok(params) => {
            let result = state.auth_service.sign_up(params).await;

            if let Err(SignUpError::TechnicalError(e)) = &result {
                error!(error = ?e, "Failed to sign up a new user");
            }

            views::auth::HandleSignupConfirmation {
                result,
                confirmation_email_resent: false,
            }
            .into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[derive(Deserialize)]
struct ResendConfirmationForm {
    user_id: Uuid,
}

async fn resend_confirmation(
    env: ExtractAppEnv,
    form: Form<ResendConfirmationForm>,
) -> impl IntoResponse {
    let result = match env
        .auth_service
        .resend_confirmation_email(form.user_id)
        .await
    {
        Ok(_) => Ok(form.user_id),
        Err(e) => Err(SignUpError::TechnicalError(e)),
    };
    views::auth::HandleSignupConfirmation {
        confirmation_email_resent: result.is_ok(),
        result,
    }
}

async fn confirm_email(env: ExtractAppEnv, Path(token): Path<String>) -> impl IntoResponse {
    match env
        .auth_service
        .confirm_email(EmailConfirmationToken { value: token })
        .await
    {
        Err(ConfirmEmailError::UserAlreadyConfirmed { user_id }) => {
            let session = Session { user_id };
            (SetSession(session, &env.config), Redirect::to("/")).into_response()
        }
        Err(e) => views::auth::ConfirmEmail { result: Err(e) }.into_response(),
        Ok(user_id) => {
            let session = Session { user_id };
            (
                SetSession(session, &env.config),
                views::auth::ConfirmEmail { result: Ok(()) },
            )
                .into_response()
        }
    }
}

pub fn auth_router() -> Router<Arc<AppEnv>> {
    Router::new()
        .route("/login", get(login).post(handle_login))
        .route("/signup", get(signup).post(handle_signup))
        .route("/resend-confirmation", post(resend_confirmation))
        .route("/confirm/:token", get(confirm_email))
}
