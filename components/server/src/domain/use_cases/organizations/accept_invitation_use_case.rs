use anyhow::Context;
use serde::Deserialize;
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;
use veil::Redact;

use crate::domain::{
    entities::{organization::ReadOrganizationError, user::CreateUserCommand},
    ports::{organization_repository::OrganizationRepository, user_repository::UserRepository},
};

#[derive(Debug, Error)]
pub enum AcceptInvitationError {
    #[error("Invitation not found")]
    InvitationNotFound,
    #[error("User cannot be empty")]
    UserCannotBeEmpty,
    #[error("User already exists and details must be empty")]
    UserAlreadyExists,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AcceptInvitationCommand {
    user_details: Option<UserDetails>,
}

#[derive(Redact, Clone, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
struct UserDetails {
    first_name: String,
    last_name: String,
    #[redact]
    password: String,
}

#[tracing::instrument(skip(organization_repository, user_repository))]
pub async fn accept_invitation_use_case(
    organization_repository: &impl OrganizationRepository,
    user_repository: &impl UserRepository,
    organization_id: Uuid,
    invitation_id: Uuid,
    command: AcceptInvitationCommand,
) -> Result<(), AcceptInvitationError> {
    // Get invitation
    let invitation = organization_repository
        .get_pending_invitation(organization_id, invitation_id)
        .await
        .map_err(|e| match e {
            ReadOrganizationError::OrganizationNotFound => {
                AcceptInvitationError::InvitationNotFound
            }
            ReadOrganizationError::TechnicalFailure(e) => {
                AcceptInvitationError::TechnicalFailure(e)
            }
        })?;

    // Check if invitee already exists
    let existing_invitee = user_repository.get_user_by_email(&invitation.email).await?;

    // Get or create user
    let user_id = match (existing_invitee, command.user_details) {
        (Some(user), None) => user.id,
        (None, Some(new_user_details)) => {
            let command = CreateUserCommand {
                first_name: new_user_details.first_name,
                last_name: new_user_details.last_name,
                email: invitation.email,
                password: new_user_details.password,
                phone_number: None,
            };
            user_repository
                .create_user(command)
                .await
                .with_context(|| "Failed to create user")?
                .id
        }
        (Some(_), Some(_)) => return Err(AcceptInvitationError::UserAlreadyExists),
        (None, None) => return Err(AcceptInvitationError::UserCannotBeEmpty),
    };

    // Add user to org
    organization_repository
        .add_an_organization_member(organization_id, user_id)
        .await
        .with_context(|| "Failed to add the user to the organization")?;

    // Grant user role
    for role in invitation.roles {
        organization_repository
            .grant_organization_role(organization_id, user_id, role)
            .await
            .with_context(|| "Failed to grant the user role")?;
    }

    // Delete invitation (ignore errors)
    let _ = organization_repository
        .delete_pending_invitation(organization_id, invitation_id)
        .await;

    Ok(())
}
