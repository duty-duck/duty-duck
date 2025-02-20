use uuid::Uuid;

use crate::domain::entities::{organization::*, user::*};

#[async_trait::async_trait]
pub trait OrganizationRepository: Clone + Send + Sync + 'static {
    /// Creates a new organization
    async fn create_organization(
        &self,
        command: CreateOrgnizationCommand,
    ) -> Result<Organization, CreateOrganizationError>;

    /// Retrieves an organization by its ID
    async fn get_organization(&self, id: Uuid) -> Result<Organization, ReadOrganizationError>;

    /// Updates an existing organization
    #[allow(unused)]
    async fn update_organization(
        &self,
        id: Uuid,
        command: UpdateOrganizationCommand,
    ) -> Result<(), WriteOrganizationError>;

    /// Lists members of an organization with pagination
    async fn list_organization_members(
        &self,
        org_id: Uuid,
        first_result_offset: u32,
        max_results: u32,
    ) -> Result<Vec<User>, ReadOrganizationError>;

    /// Adds a member to an organization
    async fn add_an_organization_member(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), WriteOrganizationError>;

    /// Removes a member from an organization
    async fn remove_an_organization_member(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), WriteOrganizationError>;

    /// Invites a new member to an organization
    async fn invite_organization_member(
        &self,
        org_id: Uuid,
        inviter_user_id: Uuid,
        invited_user_email: String,
        invited_user_role: OrganizationUserRole,
    ) -> Result<UserInvitation, WriteOrganizationError>;

    /// Deletes an organization
    #[allow(unused)]
    async fn delete_organization(&self, id: Uuid) -> Result<(), WriteOrganizationError>;

    /// Creates a new role within an organization
    async fn create_organization_role(
        &self,
        org_id: Uuid,
        role: OrganizationUserRole,
    ) -> Result<(), WriteOrganizationError>;

    /// Grants a role to a user within an organization
    async fn grant_organization_role(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        role: OrganizationUserRole,
    ) -> Result<(), WriteOrganizationRoleError>;

    /// Revokes a role from a user within an organization
    async fn revoke_organization_role(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        role: OrganizationUserRole,
    ) -> Result<(), WriteOrganizationRoleError>;

    /// Lists roles assigned to a user in an organization.
    ///
    /// # Arguments
    /// * `org_id` - Organization UUID.
    /// * `user_id` - User UUID.
    ///
    /// # Returns
    /// * `Result<Vec<OrganizationUserRole>, ReadOrganizationError>` - Vector of roles or an error.
    async fn list_organization_roles_for_user(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<OrganizationUserRole>, ReadOrganizationError>;

    /// Retrieves a pending invitation by organization and invitation ID.
    async fn get_pending_invitation(
        &self,
        org_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<UserInvitation, ReadOrganizationError>;

    /// Lists all pending invitations for an organization.
    async fn list_pending_invitations(
        &self,
        org_id: Uuid,
        first_result_offset: u32,
        max_results: u32,
    ) -> Result<Vec<UserInvitation>, ReadOrganizationError>;

    /// Deletes a pending invitation by organization and invitation ID.
    async fn delete_pending_invitation(
        &self,
        org_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<(), WriteOrganizationError>;
}
