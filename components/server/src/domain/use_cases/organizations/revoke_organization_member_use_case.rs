use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    entities::{authorization::*, organization::WriteOrganizationError},
    ports::organization_repository::OrganizationRepository,
};

#[derive(Debug, Error)]
pub enum RevokeOrganizationMemberError {
    #[error("Current user doesn't have the privilege to revoke organization members")]
    Forbidden,
    #[error("Organization not found")]
    OrganizationNotFound,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn revoke_organization_member_use_case(
    auth_context: &AuthContext,
    organization_repository: &impl OrganizationRepository,
    organization_id: Uuid,
    user_id: Uuid,
) -> Result<(), RevokeOrganizationMemberError> {
    if auth_context.active_organization_id != organization_id
        || !auth_context.can(Permission::RemoveOrganizationMember)
    {
        return Err(RevokeOrganizationMemberError::Forbidden);
    }

    match organization_repository
        .remove_an_organization_member(organization_id, user_id)
        .await
    {
        Ok(()) => Ok(()),
        Err(WriteOrganizationError::OrganizationNotFound) => {
            Err(RevokeOrganizationMemberError::OrganizationNotFound)
        }
        Err(e) => Err(RevokeOrganizationMemberError::TechnicalFailure(e.into())),
    }
}
