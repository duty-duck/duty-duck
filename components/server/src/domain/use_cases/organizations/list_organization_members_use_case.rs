use anyhow::Context;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::{
    entities::{
        authorization::*,
        organization::{OrganizationUserRole, ReadOrganizationError},
        user::User,
    },
    ports::organization_repository::OrganizationRepository,
};

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
struct ListOrganizationMembersItem {
    #[serde(flatten)]
    user: User,
    organization_roles: Vec<OrganizationUserRole>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ListOrganizationMembersResponse {
    members: Vec<ListOrganizationMembersItem>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ListOrganizationMembersParams {
    pub page_number: Option<u32>,
    pub items_per_page: Option<u32>,
}

#[derive(Debug, Error)]
pub enum ListOrganizationMembersError {
    #[error("Organization not found")]
    OrganizationNotFound,
    #[error("Current user doesn't have the privilege to list organization members")]
    Forbidden,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn list_organization_members_use_case(
    auth_context: &AuthContext,
    organization_repository: &impl OrganizationRepository,
    organization_id: Uuid,
    params: ListOrganizationMembersParams,
) -> Result<ListOrganizationMembersResponse, ListOrganizationMembersError> {
    if auth_context.active_organization_id != organization_id
        || !auth_context.can(Permission::ListOrganizationMembers)
    {
        return Err(ListOrganizationMembersError::Forbidden);
    }

    let items_per_page = params.items_per_page.unwrap_or(10).min(50);
    let page_number = params.page_number.unwrap_or(1);

    let users = match organization_repository
        .list_organization_members(
            organization_id,
            items_per_page * (page_number - 1),
            items_per_page,
        )
        .await
    {
        Ok(users) => users,
        Err(ReadOrganizationError::OrganizationNotFound) => {
            return Err(ListOrganizationMembersError::OrganizationNotFound)
        }
        Err(ReadOrganizationError::TechnicalFailure(e)) => {
            return Err(ListOrganizationMembersError::TechnicalFailure(e))
        }
    };

    let mut items = Vec::new();
    for user in users {
        let organization_roles = organization_repository
            .list_organization_roles_for_user(organization_id, user.id)
            .await
            .with_context(|| "Failed to get organization roles for user")?;
        items.push(ListOrganizationMembersItem {
            user,
            organization_roles,
        });
    }

    Ok(ListOrganizationMembersResponse { members: items })
}
