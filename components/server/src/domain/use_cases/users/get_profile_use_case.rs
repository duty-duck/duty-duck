use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        organization::{Organization, OrganizationUserRole, ReadOrganizationError},
        user::User,
    },
    ports::{organization_repository::OrganizationRepository, user_repository::UserRepository},
};

#[derive(Debug, Error)]
pub enum GetProfileError {
    #[error("the user does not exist")]
    NotFound,
    #[error("Failed to update user profile: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct GetProfileResponse {
    user: User,
    permissions: Vec<Permission>,
    active_organization: Option<Organization>,
    organization_roles: Vec<OrganizationUserRole>,
}

#[tracing::instrument(skip(auth_context, organization_repository, user_repository), err)]
pub async fn get_user_profile(
    auth_context: &AuthContext,
    organization_repository: &impl OrganizationRepository,
    user_repository: &impl UserRepository,
) -> Result<GetProfileResponse, GetProfileError> {
    tracing::debug!(user_id = ?auth_context.active_user_id, "Retrieveing current user profile");

    let organization = match auth_context.active_organization_id {
        Some(id) => match organization_repository.get_organization(id).await {
            Ok(organization) => Some(organization),
            Err(ReadOrganizationError::OrganizationNotFound) => {
                return Err(GetProfileError::NotFound)
            }
            Err(ReadOrganizationError::TechnicalFailure(e)) => {
                return Err(GetProfileError::TechnicalFailure(e))
            }
        },
        None => None,
    };

    let user = match user_repository
        .get_user(auth_context.active_user_id, false)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => return Err(GetProfileError::NotFound),
        Err(e) => return Err(GetProfileError::TechnicalFailure(e)),
    };

    let organization_roles = match auth_context.active_organization_id {
        Some(org_id) => {
            match organization_repository
                .list_organization_roles_for_user(org_id, user.id)
                .await
            {
                Ok(roles) => roles,
                Err(ReadOrganizationError::OrganizationNotFound) => {
                    return Err(GetProfileError::NotFound)
                }
                Err(ReadOrganizationError::TechnicalFailure(e)) => {
                    return Err(GetProfileError::TechnicalFailure(e))
                }
            }
        }
        None => vec![],
    };

    let response = GetProfileResponse {
        active_organization: organization,
        organization_roles,
        permissions: Permission::iter_variants()
            .filter(|p| auth_context.can(*p))
            .collect(),
        user,
    };

    Ok(response)
}
