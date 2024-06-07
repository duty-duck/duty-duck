use std::str::FromStr;

use crate::services::auth::email_confirmation::ConfirmEmailError;
use crate::services::auth::{LoginError, SignUpParams, SignUpResult};
use crate::views::filters;
use askama_axum::*;
use email_address::EmailAddress;
use itertools::Itertools;
use serde::Deserialize;
use zxcvbn::zxcvbn;

/// The login page
#[derive(Template, Default)]
#[template(path = "auth/login.html")]
pub struct LogInPage {
    pub form: LogInFormData,
    pub error: Option<LoginError>,
    pub confirmation_email_sent: bool,
}

/// The complete signup page
#[derive(Template, Default)]
#[template(path = "auth/signup-page.html")]
pub struct SignUpPage {
    form: SignupFormData,
    error: Option<String>,
    error_field: Option<&'static str>,
}

/// A page returned as a result from the handle_signup route
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
    #[allow(clippy::result_large_err)]
    pub fn validate(self) -> Result<SignUpParams, SignUpPage> {
        let name = self.name.trim().to_string();
        if name.is_empty() {
            return Err(SignUpPage {
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
                return Err(SignUpPage {
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
            return Err(SignUpPage {
                error: Some("The password field is mandatory to sign up".to_string()),
                error_field: Some("password"),
                form: SignupFormData {
                    name,
                    email: self.email,
                    ..Default::default()
                },
            });
        }
        let entropy = zxcvbn(&password, &[&name, &self.email]);
        if entropy.score() < zxcvbn::Score::Three {
            let message = match entropy.feedback() {
                Some(feedback) if !feedback.suggestions().is_empty() => {
                    let suggestions = feedback
                        .suggestions()
                        .iter()
                        .map(|s| format!("- {}", s))
                        .join("\n");
                    format!(
                        "Your password is too weak. Try the following suggestions:\n {}",
                        suggestions
                    )
                }
                _ => "Your password is too weak".to_string(),
            };
            return Err(SignUpPage {
                error: Some(message),
                error_field: Some("password"),
                form: SignupFormData {
                    name,
                    email: self.email,
                    ..Default::default()
                },
            });
        }

        if self.password_confirm != password {
            return Err(SignUpPage {
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
