use std::{str::FromStr, sync::Arc};

use anyhow::Context;
use chrono::Utc;
use tracing::warn;
use uuid::Uuid;

use crate::{
    attributes,
    domain::{
        entities::{
            organization::{
                CreateOrganizationError, Organization, OrganizationUserRole, ReadOrganizationError,
                UserInvitation, WriteOrganizationError, WriteOrganizationRoleError,
            },
            user::User,
        },
        ports::organization_repository::OrganizationRepository,
    },
    infrastructure::keycloak_client::{
        self, AttributeMap, InviteUserRequest, KeycloakClient, WriteOrganizationRequest,
    },
};

#[derive(Clone)]
pub struct OrganizationRepositoryAdapter {
    pub keycloak_client: Arc<KeycloakClient>,
}

#[async_trait::async_trait]
impl OrganizationRepository for OrganizationRepositoryAdapter {
    /// Retrieves an organization by its ID.
    #[tracing::instrument(skip(self))]
    async fn get_organization(&self, id: Uuid) -> Result<Organization, ReadOrganizationError> {
        match self.keycloak_client.get_organization(id).await {
            Ok(org) => Ok(org.try_into()?),
            Err(keycloak_client::Error::NotFound) => {
                Err(ReadOrganizationError::OrganizationNotFound)
            }
            Err(e) => {
                warn!(error = ?e, "Failed to get organization");
                Err(ReadOrganizationError::TechnicalFailure(e.into()))
            }
        }
    }

    /// Creates a new organization.
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

    /// Removes a member from an organization.
    async fn remove_an_organization_member(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), WriteOrganizationError> {
        match self
            .keycloak_client
            .remove_an_organization_member(org_id, user_id)
            .await
        {
            Ok(()) => Ok(()),
            Err(keycloak_client::Error::NotFound) => {
                Err(WriteOrganizationError::OrganizationNotFound)
            }
            Err(e) => Err(WriteOrganizationError::TechnicalFailure(e.into())),
        }
    }

    /// Adds a member to an organization.
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

    /// Invites a member to an organization.
    #[tracing::instrument(skip(self))]
    async fn invite_organization_member(
        &self,
        org_id: Uuid,
        inviter_user_id: Uuid,
        invited_user_email: String,
        invited_user_role: OrganizationUserRole,
    ) -> Result<UserInvitation, WriteOrganizationError> {
        let req = InviteUserRequest {
            email: invited_user_email,
            send: false,
            inviter_id: inviter_user_id,
            roles: vec![invited_user_role.to_string()],
            attributes: AttributeMap::default(),
        };

        match self
            .keycloak_client
            .invite_user_to_organization(org_id, &req)
            .await
        {
            Ok(invitation) => Ok(invitation.try_into()?),
            Err(keycloak_client::Error::NotFound) => {
                Err(WriteOrganizationError::OrganizationNotFound)
            }
            Err(e) => Err(WriteOrganizationError::TechnicalFailure(e.into())),
        }
    }

    /// Updates an organization's details.
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

    /// Lists members of an organization.
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

    /// Deletes an organization.
    #[tracing::instrument(skip(self))]
    async fn delete_organization(
        &self,
        id: uuid::Uuid,
    ) -> Result<(), crate::domain::entities::organization::WriteOrganizationError> {
        match self.keycloak_client.delete_organization(id).await {
            Ok(()) => Ok(()),
            Err(keycloak_client::Error::NotFound) => {
                Err(WriteOrganizationError::OrganizationNotFound)
            }
            Err(e) => Err(WriteOrganizationError::TechnicalFailure(e.into())),
        }
    }

    /// Creates a role within an organization.
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

    /// Grants a role to a user within an organization.
    #[tracing::instrument(skip(self))]
    async fn grant_organization_role(
        &self,
        org_id: uuid::Uuid,
        user_id: uuid::Uuid,
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

    /// Revokes a role from a user within an organization.
    #[tracing::instrument(skip(self))]
    async fn revoke_organization_role(
        &self,
        org_id: uuid::Uuid,
        user_id: uuid::Uuid,
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

    /// Lists roles assigned to a user within an organization.
    #[tracing::instrument(skip(self))]
    async fn list_organization_roles_for_user(
        &self,
        org_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<Vec<OrganizationUserRole>, ReadOrganizationError> {
        let roles = self
            .keycloak_client
            .list_organization_roles_for_user(org_id, user_id)
            .await
            .map_err(|e| ReadOrganizationError::TechnicalFailure(e.into()))?
            .into_iter()
            .filter_map(|role| OrganizationUserRole::from_str(&role.name).ok())
            .collect();
        Ok(roles)
    }

    /// Retrieves a pending invitation by its ID.
    #[tracing::instrument(skip(self))]
    async fn get_pending_invitation(
        &self,
        org_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<UserInvitation, ReadOrganizationError> {
        match self
            .keycloak_client
            .get_invitation_by_id(org_id, invitation_id)
            .await
        {
            Ok(invitation) => Ok(invitation.try_into()?),
            Err(keycloak_client::Error::NotFound) => {
                Err(ReadOrganizationError::OrganizationNotFound)
            }
            Err(e) => Err(ReadOrganizationError::TechnicalFailure(e.into())),
        }
    }

    /// Lists pending invitations for an organization.
    #[tracing::instrument(skip(self))]
    async fn list_pending_invitations(
        &self,
        org_id: Uuid,
        first_result_offset: u32,
        max_results: u32,
    ) -> Result<Vec<UserInvitation>, ReadOrganizationError> {
        match self
            .keycloak_client
            .list_pending_invitations(org_id, first_result_offset, max_results)
            .await
        {
            Ok(invitations) => Ok(invitations
                .into_iter()
                .map(|i| i.try_into())
                .collect::<Result<Vec<UserInvitation>, _>>()?),
            Err(e) => Err(ReadOrganizationError::TechnicalFailure(e.into())),
        }
    }

    /// Deletes a pending invitation by its ID.
    #[tracing::instrument(skip(self))]
    async fn delete_pending_invitation(
        &self,
        org_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<(), WriteOrganizationError> {
        match self
            .keycloak_client
            .remove_invitation_by_id(org_id, invitation_id)
            .await
        {
            Ok(()) => Ok(()),
            Err(keycloak_client::Error::NotFound) => {
                Err(WriteOrganizationError::OrganizationNotFound)
            }
            Err(e) => Err(WriteOrganizationError::TechnicalFailure(e.into())),
        }
    }
}

impl TryFrom<keycloak_client::Organization> for Organization {
    type Error = anyhow::Error;

    /// Converts a Keycloak organization to a domain organization.
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

impl TryFrom<keycloak_client::Invitation> for UserInvitation {
    type Error = anyhow::Error;

    /// Converts a Keycloak invitation to a domain user invitation.
    fn try_from(value: keycloak_client::Invitation) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            email: value.email,
            inviter_id: value.inviter_id,
            organization_id: value.organization_id,
            roles: value
                .roles
                .into_iter()
                .map(|r| {
                    OrganizationUserRole::from_str(&r)
                        .with_context(|| "Failed to parse organization user role")
                })
                .collect::<Result<Vec<OrganizationUserRole>, Self::Error>>()?,
        })
    }
}
