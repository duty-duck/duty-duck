use anyhow::Context;
use serde::Deserialize;
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        organization::OrganizationUserRole,
    },
    ports::organization_repository::OrganizationRepository,
};

#[derive(Debug, TS, Deserialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ChangeMemberRoleCommand {
    pub roles: Vec<OrganizationUserRole>,
}

#[derive(Error, Debug)]
pub enum ChangeMemberRoleError {
    #[error("Forbidden")]
    Forbidden,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn change_member_role_use_case(
    auth_context: &AuthContext,
    organization_repository: &impl OrganizationRepository,
    organization_id: Uuid,
    member_id: Uuid,
    command: ChangeMemberRoleCommand,
) -> Result<(), ChangeMemberRoleError> {
    if auth_context.active_organization_id != organization_id
        || !auth_context.can(Permission::EditOrganizationMember)
    {
        return Err(ChangeMemberRoleError::Forbidden);
    }

    let member_roles = organization_repository
        .list_organization_roles_for_user(organization_id, member_id)
        .await
        .with_context(|| "Failed to list organization roles for user")?;

    let roles_to_revoke = member_roles
        .iter()
        .copied()
        .filter(|role| !command.roles.contains(role))
        .collect::<Vec<_>>();

    let roles_to_add = command
        .roles
        .into_iter()
        .filter(|role| !member_roles.contains(role));

    for role in roles_to_revoke {
        organization_repository
            .revoke_organization_role(organization_id, member_id, role)
            .await
            .with_context(|| "Failed to revoke organization role")?;
    }

    for role in roles_to_add {
        organization_repository
            .grant_organization_role(organization_id, member_id, role)
            .await
            .with_context(|| "Failed to grant organization role")?;
    }

    Ok(())
}
