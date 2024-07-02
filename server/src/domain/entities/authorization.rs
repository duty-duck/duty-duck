use uuid::Uuid;

use super::organization::{OrganizationRoleSet, OrganizationUserRole};

pub struct AuthContext {
    pub active_organization_id: Uuid,
    pub active_user: Uuid,
    pub active_organization_roles: OrganizationRoleSet,
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
}
