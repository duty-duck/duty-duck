use std::collections::HashMap;

use chrono::Utc;
use tracing::warn;

use crate::{
    domain::{entities::organization::CreateOrganizationError, ports::organization_repository::OrganizationRepository},
    infrastructure::keycloak_client::{self, Attribute, CreateOrganizationRequest, KeycloakClient, OrgAttributes},
};

pub struct OrganizationRepositoryAdapter {
    pub keycloak_client: KeycloakClient,
}

impl OrganizationRepository for OrganizationRepositoryAdapter {
    async fn create_organization(
        &self,
        command: crate::domain::entities::organization::CreateOrgnizationCommand,
    ) -> Result<
        crate::domain::entities::organization::Organization,
        crate::domain::entities::organization::CreateOrganizationError,
    > {
        let now = Utc::now();
        let req = CreateOrganizationRequest {
            name: format!("{}-{}", command.name, nanoid::nanoid!(8)),
            display_name: command.display_name,
            url: None,
            domains: vec![],
            attributes: OrgAttributes {
                billing_address: command.billing_address.into(),
                stripe_customer_id: command.stripe_customer_id.into(),
                created_at: Attribute::new(now),
                updated_at: Attribute::new(now),
                rest: HashMap::new(),
            },
        };
        match self.keycloak_client.create_organization(&req).await {
            Ok(_) => {
                todo!("extract new org id")
            },
            Err(keycloak_client::Error::Conflict) => {
                Err(CreateOrganizationError::OrganizationAlreadyExists)
            },
            Err(e) => {
                warn!(error = ?e, "Failed to create an organization");
                Err(CreateOrganizationError::TechnicalFailure(e.into()))
            }
        }
    }

    async fn update_organization(
        &self,
        id: uuid::Uuid,
        command: crate::domain::entities::organization::UpdateOrganizationCommand,
    ) -> Result<
        crate::domain::entities::organization::Organization,
        crate::domain::entities::organization::WriteOrganizationError,
    > {
        todo!()
    }

    async fn list_organization_members(
        &self,
        org_id: uuid::Uuid,
        first_result_offset: u32,
        max_results: u32,
    ) -> Result<
        Vec<crate::domain::entities::user::User>,
        crate::domain::entities::organization::ReadOrganizationError,
    > {
        todo!()
    }

    async fn delete_organization(
        &self,
        id: uuid::Uuid,
    ) -> Result<(), crate::domain::entities::organization::WriteOrganizationError> {
        todo!()
    }

    async fn create_organization_role(
        &self,
        org_id: uuid::Uuid,
        role: crate::domain::entities::organization::OrganizationUserRole,
    ) -> Result<(), crate::domain::entities::organization::WriteOrganizationError> {
        todo!()
    }

    async fn grant_organization_role(
        &self,
        user_id: uuid::Uuid,
        org_id: uuid::Uuid,
        role: crate::domain::entities::organization::OrganizationUserRole,
    ) -> Result<(), crate::domain::entities::organization::WriteOrganizationRoleError> {
        todo!()
    }

    async fn revoke_organization_role(
        &self,
        user_id: uuid::Uuid,
        org_id: uuid::Uuid,
        role: crate::domain::entities::organization::OrganizationUserRole,
    ) -> Result<(), crate::domain::entities::organization::WriteOrganizationRoleError> {
        todo!()
    }
}
