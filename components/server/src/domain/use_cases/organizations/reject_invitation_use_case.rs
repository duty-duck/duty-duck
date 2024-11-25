use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    entities::organization::WriteOrganizationError,
    ports::organization_repository::OrganizationRepository,
};

#[derive(Debug, Error)]
pub enum RejectInvitationError {
    #[error("Invitation not found")]
    InvitationNotFound,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[tracing::instrument(skip(organization_repository))]
pub async fn reject_invitation_use_case(
    organization_repository: &impl OrganizationRepository,
    organization_id: Uuid,
    invitation_id: Uuid,
) -> Result<(), RejectInvitationError> {
    match organization_repository
        .delete_pending_invitation(organization_id, invitation_id)
        .await
    {
        Ok(_) => Ok(()),
        Err(WriteOrganizationError::OrganizationNotFound) => {
            Err(RejectInvitationError::InvitationNotFound)
        }
        Err(e) => Err(RejectInvitationError::TechnicalFailure(e.into())),
    }
}
