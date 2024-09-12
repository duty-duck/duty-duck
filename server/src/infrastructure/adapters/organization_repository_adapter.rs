use std::sync::Arc;

use anyhow::Context;
use chrono::Utc;
use tracing::warn;
use uuid::Uuid;

use crate::{
    attributes,
    domain::{
        entities::{
            organization::{
                CreateOrganizationError, Organization, ReadOrganizationError,
                WriteOrganizationError, WriteOrganizationRoleError,
            },
            user::User,
        },
        ports::organization_repository::OrganizationRepository,
    },
    infrastructure::keycloak_client::{
        self, AttributeMap, KeycloakClient, WriteOrganizationRequest,
    },
};

#[derive(Clone)]
pub struct OrganizationRepositoryAdapter {
    pub keycloak_client: Arc<KeycloakClient>,
}

impl OrganizationRepository for OrganizationRepositoryAdapter {
    #[tracing::instrument(skip(self))]
    async fn create_organization(
        &self,
        command: crate::domain::entities::organization::CreateOrgnizationCommand,
    ) -> Result<
        crate::domain::entities::organization::Organization,
        crate::domain::entities::organization::CreateOrganizationError,
    > {
        let now = Utc::now();
        let req = WriteOrganizationRequest {
            realm: &self.keycloak_client.realm,
            name: format!("{}-{}", command.name, nanoid::nanoid!(8)),
            display_name: command.display_name,
            url: None,
            domains: vec![],
            attributes: attributes! {
                "billing_address".to_string() => vec![serde_json::to_string(&command.billing_address).with_context(|| "Failed to serialize address")?],
                "stripe_customer_id".to_string() => command.stripe_customer_id.into_iter().collect(),
                "created_at".to_string() => vec![now.to_string()],
                "updated_at".to_string() => vec![now.to_string()],
            },
        };
        match self.keycloak_client.create_organization(&req).await {
            Ok(org) => Ok(org.try_into()?),
            Err(keycloak_client::Error::Conflict) => {
                Err(CreateOrganizationError::OrganizationAlreadyExists)
            }
            Err(e) => {
                warn!(error = ?e, "Failed to create an organization");
                Err(CreateOrganizationError::TechnicalFailure(e.into()))
            }
        }
    }

    async fn add_an_organization_member(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), WriteOrganizationError> {
        match self
            .keycloak_client
            .add_an_organization_member(org_id, user_id)
            .await
        {
            Ok(()) => Ok(()),
            Err(keycloak_client::Error::NotFound) => {
                Err(WriteOrganizationError::OrganizationNotFound)
            }
            Err(e) => Err(WriteOrganizationError::TechnicalFailure(e.into())),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn update_organization(
        &self,
        id: uuid::Uuid,
        command: crate::domain::entities::organization::UpdateOrganizationCommand,
    ) -> Result<(), crate::domain::entities::organization::WriteOrganizationError> {
        let now = Utc::now();
        let req = WriteOrganizationRequest {
            realm: &self.keycloak_client.realm,
            name: command.name,
            display_name: command.display_name,
            url: None,
            domains: vec![],
            attributes: attributes! {
                "billing_address".to_string() => vec![serde_json::to_string(&command.billing_address).with_context(|| "Failed to serialize address")?],
                "stripe_customer_id".to_string() => command.stripe_customer_id.into_iter().collect(),
                "created_at".to_string() => vec![command.created_at.to_string()],
                "updated_at".to_string() => vec![now.to_string()],
            },
        };

        match self.keycloak_client.update_organization(id, &req).await {
            Ok(()) => Ok(()),
            Err(keycloak_client::Error::NotFound) => {
                Err(WriteOrganizationError::OrganizationNotFound)
            }
            Err(e) => Err(WriteOrganizationError::TechnicalFailure(e.into())),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn list_organization_members(
        &self,
        org_id: uuid::Uuid,
        first_result_offset: u32,
        max_results: u32,
    ) -> Result<
        Vec<crate::domain::entities::user::User>,
        crate::domain::entities::organization::ReadOrganizationError,
    > {
        match self
            .keycloak_client
            .list_organization_members(org_id, first_result_offset, max_results)
            .await
        {
            Ok(uers) => {
                let users: Result<Vec<User>, _> = uers
                    .into_iter()
                    // Exclude default phase two keycloak org admins
                    .filter(|u| match &u.email {
                        Some(e) => !e.contains("@noreply.phasetwo.io"),
                        None => true,
                    })
                    .map(|u| u.try_into())
                    .collect();
                Ok(users?)
            }
            Err(keycloak_client::Error::NotFound) => {
                Err(ReadOrganizationError::OrganizationNotFound)
            }
            Err(e) => Err(ReadOrganizationError::TechnicalFailure(e.into())),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn delete_organization(
        &self,
        id: uuid::Uuid,
    ) -> Result<(), crate::domain::entities::organization::WriteOrganizationError> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn create_organization_role(
        &self,
        org_id: uuid::Uuid,
        role: crate::domain::entities::organization::OrganizationUserRole,
    ) -> Result<(), crate::domain::entities::organization::WriteOrganizationError> {
        match self
            .keycloak_client
            .create_an_organization_role(org_id, &role.to_string())
            .await
        {
            Ok(()) => Ok(()),
            Err(keycloak_client::Error::NotFound) => {
                Err(WriteOrganizationError::OrganizationNotFound)
            }
            Err(e) => Err(WriteOrganizationError::TechnicalFailure(e.into())),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn grant_organization_role(
        &self,
        user_id: uuid::Uuid,
        org_id: uuid::Uuid,
        role: crate::domain::entities::organization::OrganizationUserRole,
    ) -> Result<(), crate::domain::entities::organization::WriteOrganizationRoleError> {
        match self
            .keycloak_client
            .grant_an_organization_role(org_id, user_id, &role.to_string())
            .await
        {
            Ok(()) => Ok(()),
            Err(keycloak_client::Error::NotFound) => {
                Err(WriteOrganizationRoleError::OrganizationOrUserNotFound)
            }
            Err(e) => Err(WriteOrganizationRoleError::TechnicalFailure(e.into())),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn revoke_organization_role(
        &self,
        user_id: uuid::Uuid,
        org_id: uuid::Uuid,
        role: crate::domain::entities::organization::OrganizationUserRole,
    ) -> Result<(), crate::domain::entities::organization::WriteOrganizationRoleError> {
        match self
            .keycloak_client
            .revoke_an_organization_role(org_id, user_id, &role.to_string())
            .await
        {
            Ok(()) => Ok(()),
            Err(keycloak_client::Error::NotFound) => {
                Err(WriteOrganizationRoleError::OrganizationOrUserNotFound)
            }
            Err(e) => Err(WriteOrganizationRoleError::TechnicalFailure(e.into())),
        }
    }
}

impl TryFrom<keycloak_client::Organization> for Organization {
    type Error = anyhow::Error;

    fn try_from(value: keycloak_client::Organization) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            name: value.name,
            display_name: value.display_name,
            stripe_customer_id: value
                .attributes
                .get("stripe_customer_id")
                .map(str::to_string),
            billing_address: value.attributes.get_json("billing_address"),
            created_at: value
                .attributes
                .get("created_at")
                .and_then(|str| str.parse().ok())
                .with_context(|| "Cannot extract created_at attribute")?,
            updated_at: value
                .attributes
                .get("updated_at")
                .and_then(|str| str.parse().ok())
                .with_context(|| "Cannot extract updated_at attribute")?,
        })
    }
}
