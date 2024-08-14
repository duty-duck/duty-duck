use serde::Serialize;
use uuid::Uuid;

use super::organization::{OrganizationRoleSet, OrganizationUserRole};

#[derive(Serialize)]
pub struct AuthContext {
    pub active_organization_id: Uuid,
    pub active_user_id: Uuid,
    pub active_organization_roles: OrganizationRoleSet,
    pub last_name: Option<String>,
    pub first_name: Option<String>
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
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Permission {
    TransferOwnershipOfOrganization,
    RemoveOrganization,
    ReadHttpMonitors,
    WriteHttpMonitors,
    ReadIncidents,
}
