use askama_axum::*;
use axum::response::IntoResponse;
use axum::{routing::get, Form, Router};
use email_address::EmailAddress;
use itertools::Itertools;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use zxcvbn::zxcvbn;

use crate::app_env::{AppEnv, ExtractAppEnv};
use crate::services::auth::{SignUpError, SignUpParams, SignUpResult};

mod templates {
    use crate::routes::filters;
    use askama_axum::*;

    /// The complete login page
    #[derive(Template)]
    #[template(path = "auth/login.html")]
    pub struct Login;

    /// The complte signup page
    #[derive(Template, Default)]
    #[template(path = "auth/signup.html")]
    pub struct Signup {
        form: super::SignupForm,
        error: Option<String>,
        error_field: Option<&'static str>,
    }

    /// A partial template returned as a result from the handle_signup route
    /// when the provided form is invalid
    #[derive(Template, Default)]
    #[template(path = "auth/signup.html", block = "content")]
    pub struct HandleSignupInvalidForm {
        pub form: super::SignupForm,
        pub error: Option<String>,
        pub error_field: Option<&'static str>,
    }

    /// A partial template returned as a result from the handle_signup route
    /// when the provided form is valid and a signup attempt has been made
    #[derive(Template)]
    #[template(path = "auth/signup-confirmation.html", block = "content")]
    pub struct HandleSignupConfirmation {
        pub result: super::SignUpResult,
    }
}

#[derive(Deserialize, Default)]
struct SignupForm {
    name: String,
    email: String,
    password: String,
    password_confirm: String,
}

impl SignupForm {
    fn validate(self) -> Result<SignUpParams, templates::HandleSignupInvalidForm> {
        let name = self.name.trim().to_string();
        if name.is_empty() {
            return Err(templates::HandleSignupInvalidForm {
                error: Some("The Full Name field is mandatory to sign up".to_string()),
                error_field: Some("name"),
                form: SignupForm {
                    name,
                    email: self.email,
                    ..Default::default()
                },
            });
        }

        let email = match EmailAddress::from_str(&self.email) {
            Ok(email) => email,
            Err(_) => {
                return Err(templates::HandleSignupInvalidForm {
                    error: Some("The e-mail field is missing or invalid".to_string()),
                    error_field: Some("email"),
                    form: SignupForm {
                        name,
                        ..Default::default()
                    },
                });
            }
        };

        let password = self.password;
        if password.is_empty() {
            return Err(templates::HandleSignupInvalidForm {
                error: Some("The password field is mandatory to sign up".to_string()),
                error_field: Some("password"),
                form: SignupForm {
                    name,
                    email: self.email,
                    ..Default::default()
                },
            });
        }
        match zxcvbn(&password, &[&name, &self.email]) {
            Err(_) => {
                return Err(templates::HandleSignupInvalidForm {
                    error: Some("Failed to verify your password's strength. Please consider using another password".to_string()),
                    error_field: Some("password"),
                    form: SignupForm {
                        name,
                        email: self.email,
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
                return Err(templates::HandleSignupInvalidForm {
                    error: Some(message),
                    error_field: Some("password"),
                    form: SignupForm {
                        name,
                        email: self.email,
                        ..Default::default()
                    },
                })
            },
            _ => ()
        }

        if self.password_confirm != password {
            return Err(templates::HandleSignupInvalidForm {
                error: Some("The password confirmation does not match the password".to_string()),
                error_field: Some("password_confirm"),
                form: SignupForm {
                    name,
                    email: self.email,
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
    templates::Login
}

async fn signup() -> impl IntoResponse {
    templates::Signup::default()
}

async fn handle_signup(state: ExtractAppEnv, form: Form<SignupForm>) -> impl IntoResponse {
    match form.0.validate() {
        Ok(params) => {
            let result = state.auth_service.sign_up(params).await;

            if let Err(SignUpError::TechnicalError(e)) = &result {
                error!(error = ?e, "Failed to sign up a new user");
            }

            templates::HandleSignupConfirmation { result }.into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub fn auth_router() -> Router<Arc<AppEnv>> {
    Router::new()
        .route("/login", get(login))
        .route("/signup", get(signup).post(handle_signup))
}
