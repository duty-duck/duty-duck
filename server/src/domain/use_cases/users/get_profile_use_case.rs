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
    active_organization: Organization,
    organization_roles: Vec<OrganizationUserRole>,
}

pub async fn get_user_profile(
    auth_context: &AuthContext,
    organization_repository: &impl OrganizationRepository,
    user_repository: &impl UserRepository,
) -> Result<GetProfileResponse, GetProfileError> {
    let organization = match organization_repository
        .get_organization(auth_context.active_organization_id)
        .await
    {
        Ok(organization) => organization,
        Err(ReadOrganizationError::OrganizationNotFound) => return Err(GetProfileError::NotFound),
        Err(ReadOrganizationError::TechnicalFailure(e)) => return Err(GetProfileError::TechnicalFailure(e)),
    };

    let user = match user_repository.get_user(auth_context.active_user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(GetProfileError::NotFound),
        Err(e) => return Err(GetProfileError::TechnicalFailure(e)),
    };


    let organization_roles = match organization_repository.list_organization_roles_for_user(organization.id, user.id).await {
        Ok(roles) => roles,
        Err(ReadOrganizationError::OrganizationNotFound) => return Err(GetProfileError::NotFound),
        Err(ReadOrganizationError::TechnicalFailure(e)) => return Err(GetProfileError::TechnicalFailure(e)),
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
