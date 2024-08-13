use serde::{Deserialize, Serialize};
use thiserror::*;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: Option<String>
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserCommand {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub phone_number: Option<String>
}

#[derive(Debug, Error)]
pub enum CreateUserError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}


#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserCommand {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub phone_number: Option<String>
}

#[derive(Debug, Error)]
pub enum UpdateUserError {
    #[error("User not found")]
    UserNotFound,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}
