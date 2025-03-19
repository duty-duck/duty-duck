use serde::Deserialize;
use thiserror::Error;
use tracing::info;
use ts_rs::TS;
use uuid::Uuid;
use veil::Redact;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        organization::{OrganizationUserRole, ReadOrganizationError, WriteOrganizationError},
    },
    ports::organization_repository::OrganizationRepository,
};

#[derive(Redact, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct InviteOrganizationMemberCommand {
    #[redact(partial)]
    pub email: String,
    pub role: OrganizationUserRole,
}

#[derive(Debug, Error)]
pub enum InviteOrganizationMemberError {
    #[error("Current user doesn't have the privilege to invite organization members")]
    Forbidden,
    #[error("Organization not found")]
    OrganizationNotFound,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn invite_organization_member_use_case(
    auth_context: &AuthContext,
    organization_repository: &impl OrganizationRepository,
    organization_id: Uuid,
    command: InviteOrganizationMemberCommand,
) -> Result<(), InviteOrganizationMemberError> {
    // Forbid if the user doesn't have the permission to invite organization members
    // or if the organization id is not the active organization id
    // Why pass the organization_id in the first place if we have it in the auth context?
    // Because we want design a restful api and the endpoint is /organizations/:organizationId/members
    // and it makes sense to have the organizationId in the path. In the future, privileged users might be able to
    // invite users to other organizations too.
    if !auth_context.can(Permission::InviteOrganizationMember)
        || auth_context.active_organization_id != Some(organization_id)
    {
        return Err(InviteOrganizationMemberError::Forbidden);
    }

    // Protect the organization owner role from being invited by regular users
    // The ownership of an organization can only be transfered using a dedicated use case.
    if command.role == OrganizationUserRole::Owner {
        return Err(InviteOrganizationMemberError::Forbidden);
    }

    info!(command = ?command, "Inviting user to organization");

    let _organization = match organization_repository
        .get_organization(organization_id)
        .await
    {
        Ok(organization) => organization,
        Err(ReadOrganizationError::OrganizationNotFound) => {
            return Err(InviteOrganizationMemberError::OrganizationNotFound)
        }
        Err(ReadOrganizationError::TechnicalFailure(e)) => {
            return Err(InviteOrganizationMemberError::TechnicalFailure(e))
        }
    };

    let _invitation = match organization_repository
        .invite_organization_member(
            organization_id,
            auth_context.active_user_id,
            command.email,
            command.role,
        )
        .await
    {
        Ok(invitation) => invitation,
        Err(WriteOrganizationError::OrganizationNotFound) => {
            return Err(InviteOrganizationMemberError::OrganizationNotFound)
        }
        Err(WriteOrganizationError::TechnicalFailure(e)) => {
            return Err(InviteOrganizationMemberError::TechnicalFailure(e))
        }
    };

    Ok(())
}
