use anyhow::Context;
use uuid::Uuid;

use crate::domain::{
    entities::organization::{
        CreateOrganizationError, CreateOrgnizationCommand, OrganizationRoleSet,
        OrganizationUserRole,
    },
    ports::{organization_repository::OrganizationRepository, user_repository::UserRepository},
};

/// A use case that is automatically triggered when the logged in user does not have an active organization
/// It will attempt to retrieve an organization for the user using the Keycloak API, and
/// will create a default organization if no organization exists for the current user. This way, users can
/// self-register using Keycloak's self registration flow.
///
/// Returns the id of the active organization and the associated user roles
#[tracing::instrument(skip(organization_repository, user_repository))]
pub async fn pick_or_create_default_org(
    organization_repository: &impl OrganizationRepository,
    user_repository: &impl UserRepository,
    user_id: Uuid,
) -> anyhow::Result<(Uuid, OrganizationRoleSet)> {
    let user_orgs = organization_repository
        .list_user_organizations(user_id)
        .await
        .context("Failed to list user organizations")?;

    // User has no orgs, create the first org
    if user_orgs.is_empty() {
        let user = user_repository
            .get_user(user_id, true)
            .await?
            .context("User not found")?;

        let org_name = format!("{} {}'s team", user.first_name, user.last_name);

        // Attempt to create the organization from the user's name.
        // On conflict (an org alraedy exists with that name), increment a number until there is no conflict
        let mut attempt = 1;
        let org = loop {
            let org_name = if attempt == 1 {
                org_name.clone()
            } else {
                format!("{org_name} {attempt}")
            };

            match organization_repository
                .create_organization(CreateOrgnizationCommand {
                    name: org_name.clone(),
                    display_name: org_name,
                    // TODO: create stripe customer here ?
                    stripe_customer_id: None,
                    billing_address: None,
                })
                .await
            {
                Err(CreateOrganizationError::OrganizationAlreadyExists) => {
                    attempt += 1;
                }
                Err(CreateOrganizationError::TechnicalFailure(e)) => return Err(e),
                Ok(org) => break org,
            }
        };

        // Add user to org
        organization_repository
            .add_an_organization_member(org.id, user.id)
            .await
            .context("Failed to add the user to the organization")?;

        // Create organization roles
        create_organization_roles(organization_repository, org.id)
            .await
            .context("Failed to create organization roles")?;

        // 5. Assign owner role to new user
        organization_repository
            .grant_organization_role(org.id, user.id, OrganizationUserRole::Owner)
            .await
            .context("Failed to assign organization role Owner to user")?;

        let roles_set = OrganizationRoleSet::from_roles(vec![OrganizationUserRole::Owner]);
        Ok((org.id, roles_set))
    } else {
        let user_org = user_orgs.first().unwrap();
        let user_org_roles = organization_repository
            .list_organization_roles_for_user(user_org.id, user_id)
            .await
            .context("Failed to retrieve user roles")?;
        let roles_set = OrganizationRoleSet::from_roles(user_org_roles);
        Ok((user_org.id, roles_set))
    }
}

async fn create_organization_roles(
    organization_repository: &impl OrganizationRepository,
    org_id: Uuid,
) -> anyhow::Result<()> {
    let roles_to_create = OrganizationUserRole::ALL_ROLES;
    for role in roles_to_create {
        organization_repository
            .create_organization_role(org_id, role)
            .await
            .context("Failed to create organization role")?;
    }
    Ok(())
}
