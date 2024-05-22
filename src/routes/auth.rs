use askama_axum::*;
use axum::{routing::get, Form, Json, Router};
use email_address::EmailAddress;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;
use zxcvbn::zxcvbn;

use crate::app_env::{AppEnv, ExtractAppEnv};
use crate::routes::filters;
use crate::services::auth::{SignUpError, SignUpParams, SignUpResult};

#[derive(Template)]
#[template(path = "auth/login.html")]
struct LoginTemplate;

#[derive(Template, Default)]
#[template(path = "auth/signup.html")]
struct SignupTemplate {
    form: SignupForm,
    error: Option<String>,
    error_field: Option<&'static str>,
}

#[derive(Template)]
#[template(path = "auth/signup-confirmation.html")]
struct SignupConfirmationTemplate {
    result: SignUpResult,
}

#[derive(Deserialize, Default)]
struct SignupForm {
    name: String,
    email: String,
    password: String,
    password_confirm: String,
}

impl SignupForm {
    fn validate(self) -> Result<SignUpParams, SignupTemplate> {
        let name = self.name.trim().to_string();
        if name.is_empty() {
            return Err(SignupTemplate {
                error: Some("The Full Name field is mandatory to sign up".to_string()),
                error_field: Some("name"),
                form: SignupForm {
                    name,
                    email: self.email,
                    ..Default::default()
                },
            });
        }

        let email = self.email.trim().to_string();
        if email.is_empty() {
            return Err(SignupTemplate {
                error: Some("The e-mail field is mandatory to sign up".to_string()),
                error_field: Some("email"),
                form: SignupForm {
                    name,
                    ..Default::default()
                },
            });
        }
        if !EmailAddress::is_valid(&email) {
            return Err(SignupTemplate {
                error: Some("The e-mail field is invalid".to_string()),
                error_field: Some("email"),
                form: SignupForm {
                    name,
                    ..Default::default()
                },
            });
        }

        let password = self.password;
        if password.is_empty() {
            return Err(SignupTemplate {
                error: Some("The password field is mandatory to sign up".to_string()),
                error_field: Some("password"),
                form: SignupForm {
                    name,
                    email,
                    ..Default::default()
                },
            });
        }
        match zxcvbn(&password, &[&name, &email]) {
            Err(_) => {
                return Err(SignupTemplate {
                    error: Some("Failed to verify your password's strength. Please consider using another password".to_string()),
                    error_field: Some("password"),
                    form: SignupForm {
                        name,
                        email,
                        ..Default::default()
                    },
                })
            },
            Ok(estimate) if estimate.score() < 3 => {
                let message = match estimate.feedback() {
                    Some(feedback) if !feedback.suggestions().is_empty() => {
                        let suggestions =  feedback.suggestions().iter().map(|s| format!("- {}", s)).join("\n");
                        format!("Your password is too weak. Try the following suggestions:\n {}", suggestions)
                    },
                    _ => "Your password is too weak".to_string()
                };
                return Err(SignupTemplate {
                    error: Some(message),
                    error_field: Some("password"),
                    form: SignupForm {
                        name,
                        email,
                        ..Default::default()
                    },
                })
            },
            _ => ()
        }

        if self.password_confirm != password {
            return Err(SignupTemplate {
                error: Some("The password confirmation does not match the password".to_string()),
                error_field: Some("password_confirm"),
                form: SignupForm {
                    name,
                    email,
                    ..Default::default()
                },
            });
        }

        Ok(SignUpParams {
            full_name: name,
            email,
            password,
        })
    }
}

async fn login() -> impl IntoResponse {
    LoginTemplate
}

async fn signup() -> impl IntoResponse {
    SignupTemplate {
        error: None,
        error_field: None,
        ..Default::default()
    }
}

async fn handle_signup(state: ExtractAppEnv, form: Form<SignupForm>) -> impl IntoResponse {
    match form.0.validate() {
        Ok(params) => {
            let result = state.auth_service.sign_up(params).await;

            if let Err(SignUpError::TechnicalError(e)) = &result {
                error!(error = ?e, "Failed to sign up a new user");
            }

            SignupConfirmationTemplate { result }.into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub fn auth_router() -> Router<Arc<AppEnv>> {
    Router::new()
        .route("/login", get(login))
        .route("/signup", get(signup).post(handle_signup))
}
