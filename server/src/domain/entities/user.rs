use serde::{Deserialize, Serialize};
use uuid::Uuid;
use thiserror::*;

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserCommand {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Error)]
pub enum CreateUserError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}
 