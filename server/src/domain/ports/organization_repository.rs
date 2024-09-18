use uuid::Uuid;

use crate::domain::entities::{organization::*, user::*};

#[async_trait::async_trait]
pub trait OrganizationRepository: Clone + Send + Sync + 'static {
    async fn create_organization(
        &self,
        command: CreateOrgnizationCommand,
    ) -> Result<Organization, CreateOrganizationError>;

    async fn get_organization(&self, id: Uuid) -> Result<Organization, ReadOrganizationError>;

    async fn update_organization(
        &self,
        id: Uuid,
        command: UpdateOrganizationCommand,
    ) -> Result<(), WriteOrganizationError>;

    async fn list_organization_members(
        &self,
        org_id: Uuid,
        first_result_offset: u32,
        max_results: u32,
    ) -> Result<Vec<User>, ReadOrganizationError>;

    async fn add_an_organization_member(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), WriteOrganizationError>;

    async fn delete_organization(&self, id: Uuid) -> Result<(), WriteOrganizationError>;

    async fn create_organization_role(
        &self,
        org_id: Uuid,
        role: OrganizationUserRole,
    ) -> Result<(), WriteOrganizationError>;

    async fn grant_organization_role(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        role: OrganizationUserRole,
    ) -> Result<(), WriteOrganizationRoleError>;

    async fn revoke_organization_role(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        role: OrganizationUserRole,
    ) -> Result<(), WriteOrganizationRoleError>;
}
