use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::*;
use ts_rs::TS;
use uuid::Uuid;
use veil::Redact;

#[derive(Redact, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    #[redact(partial)]
    pub phone_number: Option<String>,
    pub phone_number_verified: bool,
    #[ts(skip)]
    pub phone_number_otp: Option<UserPhoneOTP>,
}

/// Used to verify a phone number
#[derive(Redact, Clone, Serialize, Deserialize)]
#[redact(all)]
pub struct UserPhoneOTP {
    pub phone_number: String,
    pub code: String,
    pub expires_at: DateTime<Utc>,
}

impl UserPhoneOTP {
    pub fn new(phone_number: String) -> Self {
        let code = rand::random::<u64>() % 1_000_000;
        let code_str = format!("{:06}", code);
        Self {
            phone_number,
            code: code_str,
            expires_at: Utc::now() + Duration::from_secs(600),
        }
    }
}

#[derive(Redact, Clone)]
pub struct CreateUserCommand {
    pub first_name: String,
    pub last_name: String,
    #[redact(partial)]
    pub email: String,
    #[redact]
    pub password: String,
    #[redact(partial)]
    pub phone_number: Option<String>,
}

#[derive(Debug, Error)]
pub enum CreateUserError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Default)]
pub struct UpdateUserCommand {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub phone_number: Option<String>,
    pub phone_number_verified: Option<bool>,
    pub phone_number_otp: Option<UserPhoneOTP>,
}

#[derive(Debug, Error)]
pub enum UpdateUserError {
    #[error("User not found")]
    UserNotFound,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}
