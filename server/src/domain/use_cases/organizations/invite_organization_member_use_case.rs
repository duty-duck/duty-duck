use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;
use serde::Deserialize;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        organization::{OrganizationUserRole, WriteOrganizationError},
    },
    ports::organization_repository::OrganizationRepository,
};

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct InviteOrganizationMemberCommand {
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
    if !auth_context.can(Permission::InviteOrganizationMember)
        || auth_context.active_organization_id != organization_id
    {
        return Err(InviteOrganizationMemberError::Forbidden);
    }

    match organization_repository
        .invite_organization_member(
            organization_id,
            auth_context.active_user_id,
            command.email,
            command.role,
        )
        .await
    {
        Ok(()) => Ok(()),
        Err(WriteOrganizationError::OrganizationNotFound) => {
            Err(InviteOrganizationMemberError::OrganizationNotFound)
        }
        Err(WriteOrganizationError::TechnicalFailure(e)) => {
            Err(InviteOrganizationMemberError::TechnicalFailure(e))
        }
    }
}
