use custom_derive::custom_derive;
use enum_derive::*;
use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use super::organization::{OrganizationRoleSet, OrganizationUserRole};

#[derive(Serialize)]
pub struct AuthContext {
    pub active_organization_id: Uuid,
    pub active_user_id: Uuid,
    pub active_organization_roles: OrganizationRoleSet,
    pub last_name: Option<String>,
    pub first_name: Option<String>,
}

impl AuthContext {
    pub fn can(&self, permission: Permission) -> bool {
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
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub enum Permission {
        /// Permission to transfer ownership of the organization
        TransferOwnershipOfOrganization,
        /// Permission to invite a new member to the organization
        InviteOrganizationMember,
        /// Permission to remove a member from the organization
        RemoveOrganizationMember,
        /// Permission to list all members of the organization
        ListOrganizationMembers,
        /// Permission to edit a member's details within the organization
        EditOrganizationMember,
        /// Permission to remove the organization
        RemoveOrganization,
        /// Permission to read HTTP monitors
        ReadHttpMonitors,
        /// Permission to write HTTP monitors
        WriteHttpMonitors,
        /// Permission to read incidents
        ReadIncidents,
        /// Permission to list all invitations for the organization
        ListOrganizationInvitations,
        /// Comment incidents
        CommentIncidents,
        /// Edit incidents (acknowledge, resolve, etc.)
        EditIncidents,
    }
}
