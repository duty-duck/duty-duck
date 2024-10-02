use serde::Serialize;
use thiserror::*;
use ts_rs::TS;
use uuid::Uuid;
use veil::Redact;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: Option<String>,
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

#[derive(Debug, Clone)]
pub struct UpdateUserCommand {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Error)]
pub enum UpdateUserError {
    #[error("User not found")]
    UserNotFound,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}
