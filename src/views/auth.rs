use std::str::FromStr;

use crate::services::auth::email_confirmation::ConfirmEmailError;
use crate::services::auth::{LoginError, SignUpParams, SignUpResult};
use crate::views::filters;
use askama_axum::*;
use email_address::EmailAddress;
use itertools::Itertools;
use serde::Deserialize;
use uuid::Uuid;
use zxcvbn::zxcvbn;

/// The complete login page
#[derive(Template, Default)]
#[template(path = "auth/login.html")]
pub struct LogInPage {
    pub form: LogInFormData,
    pub error: Option<LoginError>,
    pub confirmation_email_sent: bool,
}

/// A partial view of the log in page, intended to display a feedback
/// when a login attempt fails
#[derive(Template, Default)]
#[template(path = "auth/login-form.html")]
pub struct LogInForm {
    pub form: LogInFormData,
    pub error: Option<LoginError>,
    pub confirmation_email_sent: bool,
}

/// The complete sign up page
#[derive(Template, Default)]
#[template(path = "auth/signup-page.html")]
pub struct SignUpPage {
    form: SignupFormData,
    error: Option<String>,
    error_field: Option<&'static str>,
}

/// A partial template returned as a result from the handle_signup route
/// when the provided form is invalid
#[derive(Template, Default)]
#[template(path = "auth/signup-form.html")]
pub struct SignUpForm {
    pub form: SignupFormData,
    pub error: Option<String>,
    pub error_field: Option<&'static str>,
}

/// A partial template returned as a result from the handle_signup route
/// when the provided form is valid and a signup attempt has been made
#[derive(Template)]
#[template(path = "auth/send-email-confirmation-button.html")]
pub struct SendEmailConfirmationButton {
    pub user_id: Uuid,
    pub confirmation_email_sent: bool,
}

/// A partial template returned as a result from the handle_signup route
/// when the provided form is valid and a signup attempt has been made
#[derive(Template)]
#[template(path = "auth/signup-confirmation.html")]
pub struct HandleSignupConfirmation {
    pub result: SignUpResult,
    pub confirmation_email_sent: bool,
}

/// A template returned as a result from the confirm_email route
#[derive(Template)]
#[template(path = "auth/email-confirmation.html")]
pub struct ConfirmEmail {
    pub result: Result<(), ConfirmEmailError>,
}

#[derive(Deserialize, Default)]
pub struct SignupFormData {
    name: String,
    email: String,
    password: String,
    password_confirm: String,
}

impl SignupFormData {
    pub fn validate(self) -> Result<SignUpParams, SignUpForm> {
        let name = self.name.trim().to_string();
        if name.is_empty() {
            return Err(SignUpForm {
                error: Some("The Full Name field is mandatory to sign up".to_string()),
                error_field: Some("name"),
                form: SignupFormData {
                    name,
                    email: self.email,
                    ..Default::default()
                },
            });
        }

        let email = match EmailAddress::from_str(&self.email) {
            Ok(email) => email,
            Err(_) => {
                return Err(SignUpForm {
                    error: Some("The e-mail field is missing or invalid".to_string()),
                    error_field: Some("email"),
                    form: SignupFormData {
                        name,
                        ..Default::default()
                    },
                });
            }
        };

        let password = self.password;
        if password.is_empty() {
            return Err(SignUpForm {
                error: Some("The password field is mandatory to sign up".to_string()),
                error_field: Some("password"),
                form: SignupFormData {
                    name,
                    email: self.email,
                    ..Default::default()
                },
            });
        }
        match zxcvbn(&password, &[&name, &self.email]) {
            Err(_) => {
                return Err(SignUpForm {
                    error: Some("Failed to verify your password's strength. Please consider using another password".to_string()),
                    error_field: Some("password"),
                    form: SignupFormData {
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
                return Err(SignUpForm {
                    error: Some(message),
                    error_field: Some("password"),
                    form: SignupFormData {
                        name,
                        email: self.email,
                        ..Default::default()
                    },
                })
            },
            _ => ()
        }

        if self.password_confirm != password {
            return Err(SignUpForm {
                error: Some("The password confirmation does not match the password".to_string()),
                error_field: Some("password_confirm"),
                form: SignupFormData {
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

#[derive(Deserialize, Default)]
pub struct LogInFormData {
    pub email: String,
    pub password: String,
}
