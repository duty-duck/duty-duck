use axum::extract::{Path, Request};
use axum::middleware::{self, Next};
use axum::response::{AppendHeaders, IntoResponse, Redirect};
use axum::routing::post;
use axum::{routing::get, Form, Router};
use serde::Deserialize;
use std::time::Duration;
use tracing::error;
use uuid::Uuid;

use crate::app_env::{AppEnv, ExtractAppEnv};
use crate::services::auth::email_confirmation::{ConfirmEmailError, EmailConfirmationToken};
use crate::services::auth::SignUpError;
use crate::session::{ClearSession, Session, SetSession};
use crate::views;

async fn login(current_session: Option<Session>) -> impl IntoResponse {
    if current_session.is_some() {
        return Redirect::to("/dashboard").into_response();
    }
    views::auth::LogInPage::default().into_response()
}

async fn signup(current_session: Option<Session>) -> impl IntoResponse {
    if current_session.is_some() {
        return Redirect::to("/dashboard").into_response();
    }
    views::auth::SignUpPage::default().into_response()
}

async fn handle_login(
    env: ExtractAppEnv,
    Form(form): Form<views::auth::LogInFormData>,
) -> impl IntoResponse {
    match env.auth_service.log_in(&form.email, &form.password).await {
        Ok(user) => {
            let session = Session::new(user.id);
            (
                SetSession(session, &env.config),
                AppendHeaders([("HX-Location", "/dashboard")]),
            )
                .into_response()
        }
        Err(e) => {
            // Delay the response for a few seconds to avoid malicious users from making too many attempts
            tokio::time::sleep(Duration::from_secs(2)).await;
            views::auth::LogInForm {
                error: Some(e),
                ..Default::default()
            }
            .into_response()
        }
    }
}

async fn handle_logout() -> impl IntoResponse {
    (ClearSession, AppendHeaders([("HX-Location", "/")]))
}

async fn handle_signup(
    state: ExtractAppEnv,
    form: Form<views::auth::SignupFormData>,
) -> impl IntoResponse {
    match form.0.validate() {
        Ok(params) => {
            let result = state.auth_service.sign_up(params).await;

            if let Err(SignUpError::TechnicalError(e)) = &result {
                error!(error = ?e, "Failed to sign up a new user");
            }

            views::auth::HandleSignupConfirmation {
                result,
                confirmation_email_sent: false,
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
    // Add an artificial dealy to prevent malicious users from spamming this endpoints
    tokio::time::sleep(Duration::from_secs(1)).await;

    match env
        .auth_service
        .resend_confirmation_email(form.user_id)
        .await
    {
        Ok(_) => views::auth::SendEmailConfirmationButton {
            confirmation_email_sent: true,
            user_id: form.user_id,
        }
        .into_response(),
        Err(_) => "Something went wrong".into_response(),
    }
}

async fn confirm_email(env: ExtractAppEnv, Path(token): Path<String>) -> impl IntoResponse {
    match env
        .auth_service
        .confirm_email(EmailConfirmationToken { value: token })
        .await
    {
        Err(ConfirmEmailError::UserAlreadyConfirmed { user_id }) => {
            let session = Session::new(user_id);
            (SetSession(session, &env.config), Redirect::to("/")).into_response()
        }
        Err(e) => views::auth::ConfirmEmail { result: Err(e) }.into_response(),
        Ok(user_id) => {
            let session = Session::new(user_id);
            (
                SetSession(session, &env.config),
                views::auth::ConfirmEmail { result: Ok(()) },
            )
                .into_response()
        }
    }
}

pub fn auth_router() -> Router<AppEnv> {
    Router::new()
        .route("/login", get(login).post(handle_login))
        .route("/logout", post(handle_logout))
        .route("/signup", get(signup).post(handle_signup))
        .route("/resend-confirmation", post(resend_confirmation))
        .route("/confirm/:token", get(confirm_email))
}
