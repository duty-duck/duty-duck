use std::str::FromStr;

use anyhow::Context;
use email_address::EmailAddress;
use futures::TryFutureExt;
use lazy_static::lazy_static;
use nanoid::nanoid;
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;
use tokio::try_join;
use tracing::info;
use ts_rs::TS;
use uuid::Uuid;
use veil::Redact;
use zxcvbn::{zxcvbn, Entropy, Score};

use crate::domain::{
    entities::{
        organization::{CreateOrgnizationCommand, OrganizationUserRole},
        user::{CreateUserCommand, CreateUserError},
    },
    ports::{organization_repository::OrganizationRepository, user_repository::UserRepository},
};

lazy_static! {
    static ref WHITESPACE_REGEX: Regex = Regex::new("\\s+").unwrap();
}

#[derive(TS, Deserialize, Redact)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SignUpCommand {
    pub organization_name: String,
    pub first_name: String,
    pub last_name: String,
    #[redact(partial)]
    pub email: String,
    #[redact]
    pub password: String,
}

#[derive(Debug, Error)]
pub enum SignUpError {
    #[error("e-mail is invalid")]
    InvalidEmail,
    #[error("password too weak")]
    PasswordTooWeak,
    #[error("user already exists")]
    UserAlreadyExists,
    #[error("technical failure: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

#[tracing::instrument(skip(organization_repository, user_repository))]
pub async fn sign_up(
    organization_repository: &impl OrganizationRepository,
    user_repository: &impl UserRepository,
    command: SignUpCommand,
) -> Result<(), SignUpError> {
    // 0: Check e-mail and password validity
    let email = EmailAddress::from_str(&command.email)
        .map_err(|_| SignUpError::InvalidEmail)?
        .to_string();
    let password_entropy = zxcvbn(
        &command.password,
        &[
            &command.first_name,
            &command.last_name,
            &command.organization_name,
        ],
    );
    let password = if password_entropy.score() >= Score::Three {
        command.password
    } else {
        return Err(SignUpError::PasswordTooWeak);
    };

    // 1-2. Create user and organization in parallel
    let create_user = user_repository
        .create_user(CreateUserCommand {
            first_name: command.first_name,
            last_name: command.last_name,
            email,
            password,
            phone_number: None
        })
        .map_err(|e| match e {
            CreateUserError::UserAlreadyExists => SignUpError::UserAlreadyExists,
            e => SignUpError::TechnicalFailure(e.into()),
        });
    let create_org = organization_repository
        .create_organization(CreateOrgnizationCommand {
            name: format!(
                "{}_{}",
                WHITESPACE_REGEX.replace_all(&command.organization_name, "_"),
                nanoid!(10)
            ),
            display_name: command.organization_name,
            // TODO: create stripe customer when signing up
            stripe_customer_id: None,
            billing_address: None,
        })
        .map_err(|e| SignUpError::TechnicalFailure(e.into()));
    let (user, org) = try_join!(create_user, create_org)?;

    // 4. Add user to org
    organization_repository
        .add_an_organization_member(org.id, user.id)
        .await
        .with_context(|| "Failed to add the user to the organization")?;

    // 4. Create organization roles
    create_organization_roles(organization_repository, org.id)
        .await
        .with_context(|| "Failed to create organization roles")?;

    // 5. Assign owner role to new user
    organization_repository
        .grant_organization_role(user.id, org.id, OrganizationUserRole::Owner)
        .await
        .with_context(|| "Failed to assign organization role Owner to user")?;

    // 6. Return
    info!(organization = ?org, user = ?user, "Signed up a new user");
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckPasswordStrengthCommand {
    password: String,
    first_name: String,
    last_name: String,
}

pub fn check_password_strength(command: CheckPasswordStrengthCommand) -> Entropy {
    zxcvbn(
        &command.password,
        &[&command.first_name, &command.last_name],
    )
}

async fn create_organization_roles(
    organization_repository: &impl OrganizationRepository,
    org_id: Uuid,
) -> Result<(), SignUpError> {
    let roles_to_create = OrganizationUserRole::ALL_ROLES;
    for role in roles_to_create {
        organization_repository
            .create_organization_role(org_id, role)
            .await
            .map_err(|e| SignUpError::TechnicalFailure(e.into()))?;
    }
    Ok(())
}
