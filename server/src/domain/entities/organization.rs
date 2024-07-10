use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    pub name: String,
    pub display_name: String,
    pub stripe_customer_id: Option<String>,
    pub billing_address: Option<Address>,
    pub created_at: DateTime<Utc>,
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
    #[error("Organization or user not found")]
    OrganizationOrUserNotFound,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

impl FromStr for OrganizationUserRole {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::String(s.to_string()))
    }
}

impl std::fmt::Display for OrganizationUserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl OrganizationUserRole {
    pub const ALL_ROLES: [Self; 6] = [
        Self::Reporter,
        Self::Editor,
        Self::MemberInviter,
        Self::MemberManager,
        Self::Administrator,
        Self::Owner,
    ];
}

/// A list of organization roles assigned to a user
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct OrganizationRoleSet {
    roles: Vec<OrganizationUserRole>,
}

impl OrganizationRoleSet {
    /// Returns whether this [OrganizationRoleSet] contains the specified role.
    /// This function implements a hierarchy of roles: for all roles A and B where A includes all the privileges of B, then contains(A) implies contains(B).
    ///
    /// Owner includes all roles
    /// Administrator includes all roles except Owner
    /// Editor includes Reporter
    /// MemberManager includes MemberInviter
    #[inline]
    pub fn contains(&self, role: OrganizationUserRole) -> bool {
        match role {
            OrganizationUserRole::Owner => self.roles.contains(&role),
            OrganizationUserRole::Reporter => self.contains_one_of(&[
                OrganizationUserRole::Owner,
                OrganizationUserRole::Administrator,
                OrganizationUserRole::Editor,
                role,
            ]),
            OrganizationUserRole::MemberInviter => self.contains_one_of(&[
                OrganizationUserRole::Owner,
                OrganizationUserRole::Administrator,
                OrganizationUserRole::MemberManager,
                role,
            ]),
            _ => self.contains_one_of(&[
                OrganizationUserRole::Owner,
                OrganizationUserRole::Administrator,
                role,
            ]),
        }
    }

    #[inline]
    fn contains_one_of(&self, roles: &[OrganizationUserRole]) -> bool {
        for role in &self.roles {
            for r in roles {
                if r == role {
                    return true;
                }
            }
        }
        false
    }
}

impl<I: IntoIterator<Item = String>> From<I> for OrganizationRoleSet {
    fn from(value: I) -> Self {
        Self {
            roles: value.into_iter().filter_map(|r| r.parse().ok()).collect(),
        }
    }
}
