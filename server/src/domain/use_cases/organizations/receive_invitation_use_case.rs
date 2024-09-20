use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::{
    entities::{
        organization::{Organization, UserInvitation},
        user::User,
    },
    ports::{organization_repository::OrganizationRepository, user_repository::UserRepository},
};

#[derive(Error, Debug)]
pub enum ReceiveInvitationError {
    #[error("Organization not found")]
    OrganizationNotFound,
    #[error("Invitation not found")]
    InvitationNotFound,
    #[error("User not found")]
    InviterUserNotFound,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ReceiveInvitationResponse {
    /// The invitation contains information about the invitee and the role they are invited to.
    pub invitation: UserInvitation,
    /// The inviter is the user that invited the invitee to the organization.
    pub inviter: User,
    pub organization: Organization,
    /// The invitee may or may not exist in the system, since users can belong to multiple organizations.
    pub invitee: Option<User>,
}

/// Receive an invitation to an organization.
/// Notice how the auth context is not required for this use case.
#[tracing::instrument(skip(organization_repository, user_repository))]
pub async fn receive_invitation_use_case(
    organization_repository: &impl OrganizationRepository,
    user_repository: &impl UserRepository,
    organization_id: Uuid,
    invitation_id: Uuid,
) -> Result<ReceiveInvitationResponse, ReceiveInvitationError> {
    let organization = organization_repository
        .get_organization(organization_id)
        .await
        .map_err(|_| ReceiveInvitationError::OrganizationNotFound)?;

    let invitation = organization_repository
        .get_pending_invitation(organization_id, invitation_id)
        .await
        .map_err(|_| ReceiveInvitationError::InvitationNotFound)?;

    let inviter = match user_repository.get_user(invitation.inviter_id).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(ReceiveInvitationError::InviterUserNotFound),
        Err(e) => return Err(ReceiveInvitationError::TechnicalFailure(e)),
    };

    let invitee = user_repository
        .get_user_by_email(&invitation.email)
        .await
        .map_err(ReceiveInvitationError::TechnicalFailure)?;

    Ok(ReceiveInvitationResponse {
        invitation,
        inviter,
        invitee,
        organization,
    })
}
