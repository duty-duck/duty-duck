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
        }
    }
}

custom_derive! {
    #[derive(Clone, Copy, Debug, EnumDisplay, IterVariants(GetVariants))]
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub enum Permission {
        TransferOwnershipOfOrganization,
        InviteOrganizationMember,
        RemoveOrganizationMember,
        ListOrganizationMembers,
        EditOrganizationMember,
        RemoveOrganization,
        ReadHttpMonitors,
        WriteHttpMonitors,
        ReadIncidents,
    }
}
