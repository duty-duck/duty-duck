use axum::extract::Path;
use axum::response::{IntoResponse, Redirect};
use axum::routing::post;
use axum::{routing::get, Form, Router};
use serde::Deserialize;
use std::time::Duration;
use uuid::Uuid;

use crate::app_env::{AppEnv, ExtractAppEnv};
use crate::http_utils::session::*;
use crate::http_utils::tenant_based_routing::CurrentTenant;
use crate::services::auth::email_confirmation::{ConfirmEmailError, EmailConfirmationToken};
use crate::views;

async fn login(
    CurrentTenant(_): CurrentTenant,
    current_session: Option<Session>,
) -> impl IntoResponse {
    if current_session.is_some() {
        return Redirect::to("/dashboard").into_response();
    }
    views::auth::LogInPage::default().into_response()
}

async fn handle_login(
    env: ExtractAppEnv,
    CurrentTenant(tenant): CurrentTenant,
    Form(form): Form<views::auth::LogInFormData>,
) -> impl IntoResponse {
    match env
        .auth_service
        .log_in(tenant.id, &form.email, &form.password)
        .await
    {
        Ok(user) => {
            let session = Session::new(user.id);
            (SetSession(session, &env.config), Redirect::to("/dashboard")).into_response()
        }
        Err(e) => {
            // Delay the response for a few seconds to avoid malicious users from making too many attempts
            tokio::time::sleep(Duration::from_secs(2)).await;
            views::auth::LogInPage {
                error: Some(e),
                ..Default::default()
            }
            .into_response()
        }
    }
}

async fn handle_logout() -> impl IntoResponse {
    (ClearSession, Redirect::to("/"))
}


#[derive(Deserialize)]
struct ResendConfirmationForm {
    user_id: Uuid,
}

async fn resend_confirmation(
    env: ExtractAppEnv,
    CurrentTenant(tenant): CurrentTenant,
    form: Form<ResendConfirmationForm>,
) -> impl IntoResponse {
    // Add an artificial dealy to prevent malicious users from spamming this endpoints
    tokio::time::sleep(Duration::from_secs(1)).await;

    match env
        .auth_service
        .resend_confirmation_email(tenant.id, form.user_id)
        .await
    {
        Ok(_) => views::auth::HandleSignupConfirmation {
            confirmation_email_sent: true,
            result: Err(SignUpError::UnconfirmedUserAlreadyExists {
                user_id: form.user_id,
            }),
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
        .route("/logout", get(handle_logout))
        .route("/resend-confirmation", post(resend_confirmation))
        .route("/confirm/:token", get(confirm_email))
}
