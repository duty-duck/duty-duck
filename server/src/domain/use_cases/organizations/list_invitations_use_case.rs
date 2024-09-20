use thiserror::Error;
use uuid::Uuid;
use serde::Deserialize;
use ts_rs::TS;

use crate::domain::{entities::{authorization::{AuthContext, Permission}, organization::{ReadOrganizationError, UserInvitation}}, ports::organization_repository::OrganizationRepository};

#[derive(Error, Debug)]

pub enum ListInvitationsError{
    #[error("Organization not found")]
    OrganizationNotFound,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
} 

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ListInvitationsParams {
    pub page_number: u32,
    pub items_per_page: u32,
}

pub async fn list_invitations_use_case(
    auth_context: &AuthContext,
    organization_repository: &impl OrganizationRepository,
    organization_id: Uuid,
    params: ListInvitationsParams,
) -> Result<Vec<UserInvitation>, ListInvitationsError> {
    if !auth_context.can(Permission::ListOrganizationInvitations) || organization_id != auth_context.active_organization_id {
        return Err(ListInvitationsError::PermissionDenied);
    }

    let first_result_offset = (params.page_number - 1) * params.items_per_page;
    let max_results = params.items_per_page;

    match organization_repository.list_pending_invitations(organization_id, first_result_offset, max_results).await {
        Ok(invitations) => Ok(invitations),
        Err(ReadOrganizationError::OrganizationNotFound) => Err(ListInvitationsError::OrganizationNotFound),
        Err(e) => Err(ListInvitationsError::TechnicalFailure(e.into())),
    }
}