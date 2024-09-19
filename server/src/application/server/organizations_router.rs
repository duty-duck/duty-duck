use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{entities::authorization::AuthContext, use_cases::organizations},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use tracing::warn;
use uuid::Uuid;

pub(crate) fn organizations_router() -> Router<ApplicationState> {
    Router::new()
        .route("/:organizations_id/members", get(list_members_handler))
        .route(
            "/:organization_id/members/invite",
            post(invite_member_handler),
        )
        .route(
            "/:organization_id/members/:member_id",
            delete(remove_member_handler),
        )
        .route(
            "/:organization_id/members/:member_id/roles",
            put(change_member_role_handler),
        )
}

async fn list_members_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(organization_id): Path<Uuid>,
    Query(params): Query<organizations::ListOrganizationMembersParams>,
) -> impl IntoResponse {
    match organizations::list_organization_members_use_case(
        &auth_context,
        &app_state.adapters.organization_repository,
        organization_id,
        params,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(organizations::ListOrganizationMembersError::OrganizationNotFound) => {
            StatusCode::NOT_FOUND.into_response()
        }
        Err(organizations::ListOrganizationMembersError::Forbidden) => {
            StatusCode::FORBIDDEN.into_response()
        }
        Err(organizations::ListOrganizationMembersError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting organization members from the database");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn invite_member_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(organization_id): Path<Uuid>,
    Json(command): Json<organizations::InviteOrganizationMemberCommand>,
) -> impl IntoResponse {
    match organizations::invite_organization_member_use_case(
        &auth_context,
        &app_state.adapters.organization_repository,
        organization_id,
        command,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(organizations::InviteOrganizationMemberError::OrganizationNotFound) => {
            StatusCode::NOT_FOUND.into_response()
        }
        Err(organizations::InviteOrganizationMemberError::Forbidden) => {
            StatusCode::FORBIDDEN.into_response()
        }
        Err(organizations::InviteOrganizationMemberError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while inviting organization member to the database");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn remove_member_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path((organization_id, member_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match organizations::revoke_organization_member_use_case(
        &auth_context,
        &app_state.adapters.organization_repository,
        organization_id,
        member_id,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(organizations::RevokeOrganizationMemberError::OrganizationNotFound) => {
            StatusCode::NOT_FOUND.into_response()
        }
        Err(organizations::RevokeOrganizationMemberError::Forbidden) => {
            StatusCode::FORBIDDEN.into_response()
        }
        Err(organizations::RevokeOrganizationMemberError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while removing organization member from the database");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn change_member_role_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path((organization_id, member_id)): Path<(Uuid, Uuid)>,
    Json(command): Json<organizations::ChangeMemberRoleCommand>,
) -> impl IntoResponse {
    match organizations::change_member_role_use_case(
        &auth_context,
        &app_state.adapters.organization_repository,
        organization_id,
        member_id,
        command,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(organizations::ChangeMemberRoleError::Forbidden) => {
            StatusCode::FORBIDDEN.into_response()
        }
        Err(organizations::ChangeMemberRoleError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while changing organization member role");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}