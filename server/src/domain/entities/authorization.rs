use anyhow::anyhow;
use chrono::{DateTime, Utc};
use custom_derive::custom_derive;
use enum_derive::*;
use rand::Rng;
use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;
use veil::Redact;

use super::organization::{OrganizationRoleSet, OrganizationUserRole};

#[derive(Serialize)]
pub struct AuthContext {
    pub active_organization_id: Uuid,
    pub active_user_id: Uuid,
    pub active_organization_roles: OrganizationRoleSet,
    pub restricted_to_scopes: Vec<Permission>,
}

impl AuthContext {
    pub fn can(&self, permission: Permission) -> bool {
        if !self.restricted_to_scopes.is_empty() && !self.restricted_to_scopes.contains(&permission)
        {
            return false;
        }

        match permission {
            Permission::TransferOwnershipOfOrganization | Permission::RemoveOrganization => self
                .active_organization_roles
                .contains(OrganizationUserRole::Owner),
            Permission::ReadHttpMonitors => self
                .active_organization_roles
                .contains(OrganizationUserRole::Reporter),
            Permission::ReadIncidents => self
                .active_organization_roles
                .contains(OrganizationUserRole::Reporter),
            Permission::WriteHttpMonitors => self
                .active_organization_roles
                .contains(OrganizationUserRole::Editor),
            Permission::InviteOrganizationMember => self
                .active_organization_roles
                .contains(OrganizationUserRole::MemberInviter),
            Permission::RemoveOrganizationMember => self
                .active_organization_roles
                .contains(OrganizationUserRole::MemberManager),
            Permission::ListOrganizationMembers => self
                .active_organization_roles
                .contains(OrganizationUserRole::MemberManager),
            Permission::EditOrganizationMember => self
                .active_organization_roles
                .contains(OrganizationUserRole::MemberManager),
            Permission::ListOrganizationInvitations => self
                .active_organization_roles
                .contains(OrganizationUserRole::MemberInviter),
            Permission::CommentIncidents => self
                .active_organization_roles
                .contains(OrganizationUserRole::Reporter),
            Permission::EditIncidents => self
                .active_organization_roles
                .contains(OrganizationUserRole::Editor),
        }
    }
}

custom_derive! {
    #[derive(Clone, Copy, Debug, EnumDisplay, IterVariants(GetVariants))]
    #[derive(Serialize, TS, PartialEq, Eq)]
    #[derive(sqlx::Type)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    #[repr(i16)]
    pub enum Permission {
        /// Permission to transfer ownership of the organization
        TransferOwnershipOfOrganization = 1,
        /// Permission to invite a new member to the organization
        InviteOrganizationMember = 2,
        /// Permission to remove a member from the organization
        RemoveOrganizationMember = 3,
        /// Permission to list all members of the organization
        ListOrganizationMembers = 4,
        /// Permission to edit a member's details within the organization
        EditOrganizationMember = 5,
        /// Permission to remove the organization
        RemoveOrganization = 6,
        /// Permission to read HTTP monitors
        ReadHttpMonitors = 7,
        /// Permission to write HTTP monitors
        WriteHttpMonitors = 8,
        /// Permission to read incidents
        ReadIncidents = 9,
        /// Permission to list all invitations for the organization
        ListOrganizationInvitations = 10,
        /// Comment incidents
        CommentIncidents = 11,
        /// Edit incidents (acknowledge, resolve, etc.)
        EditIncidents = 12,
    }
}

impl From<i16> for Permission {
    fn from(value: i16) -> Self {
        match value {
            1 => Self::TransferOwnershipOfOrganization,
            2 => Self::InviteOrganizationMember,
            3 => Self::RemoveOrganizationMember,
            4 => Self::ListOrganizationMembers,
            5 => Self::EditOrganizationMember,
            6 => Self::RemoveOrganization,
            7 => Self::ReadHttpMonitors,
            8 => Self::WriteHttpMonitors,
            9 => Self::ReadIncidents,
            10 => Self::ListOrganizationInvitations,
            11 => Self::CommentIncidents,
            12 => Self::EditIncidents,
            _ => panic!("invalid Permission discriminant: {value}"),
        }
    }
}

#[derive(sqlx::FromRow, Clone, Redact)]
pub struct ApiAccessToken {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub label: String,
    // A hashed 256-bit key
    #[redact]
    pub secret_key: Vec<u8>,
    pub scopes: Vec<Permission>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl ApiAccessToken {
    /// Encodes a 256-bit secret key into an alphanumeric string
    pub fn encode_secret_key(secret_key: &[u8]) -> String {
        hex::encode(secret_key)
    }

    /// Decodes an alphanumeric string into a 256-bit secret key
    pub fn decode_secret_key(encoded_key: &str) -> anyhow::Result<Vec<u8>> {
        hex::decode(encoded_key).map_err(|_| anyhow!("Failed to decode secret key"))
    }

    /// Generate a random 256-bit secret key
    pub fn generate_secret_key() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..32).map(|_| rng.gen::<u8>()).collect::<Vec<_>>()
    }
}
