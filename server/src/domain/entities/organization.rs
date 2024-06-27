use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub stripe_customer_id: Option<String>,
    pub billing_address: Option<Address>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrgnizationCommand {
    pub name: String,
    pub display_name: String,
    pub stripe_customer_id: Option<String>,
    pub billing_address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrganizationCommand {
    pub display_name: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub billing_address: Option<Address>,
}

#[derive(Debug, Error)]
pub enum CreateOrganizationError {
    #[error("Organization already exists")]
    OrganizationAlreadyExists,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum WriteOrganizationError {
    #[error("Organization not found")]
    OrganizationNotFound,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum ReadOrganizationError {
    #[error("Organization not found")]
    OrganizationNotFound,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum WriteOrganizationRoleError {
    #[error("Organization not found")]
    OrganizationNotFound,
    #[error("User not found")]
    UserNotFound,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Address {
    pub line_1: String,
    pub line_2: String,
    pub city: String,
    pub state_or_province: String,
    pub postal_code: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by_user_id: Uuid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrganizationUserRole {
    /// Can read incidents and status pages but not write anything
    Reporter,
    /// Can read and write monitors, incidents etc.
    Editor,
    /// Can add new members to the organization
    MemberInviter,
    /// Can add and remove members of the organization
    MemberManager,
    /// Encompasses all other roles,
    /// Can do everything in the organization, except transfer ownership of the organization
    Administrator,
    /// Can do everything in the organization.
    /// The owner is a unique administrator of the organization whose role cannot be revoked, unless transfered to another user,
    /// so that there always exactly one organization owner
    Owner,
}
