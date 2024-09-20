use std::sync::Arc;

use anyhow::Context;
use uuid::Uuid;

use crate::domain::entities::user::*;
use crate::domain::ports::user_repository::UserRepository;
use crate::infrastructure::keycloak_client::{
    self, AttributeMap, CreateUserRequest, Credentials, KeycloakClient, UpdateUserRequest,
};

#[derive(Clone)]
pub struct UserRepositoryAdapter {
    pub keycloak_client: Arc<KeycloakClient>,
}

impl UserRepository for UserRepositoryAdapter {
    #[tracing::instrument(skip(self))]
    async fn get_user(&self, id: Uuid) -> anyhow::Result<Option<User>> {
        match self.keycloak_client.get_user_by_id(id).await {
            Ok(user) => Ok(Some(user.try_into()?)),
            Err(keycloak_client::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn get_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        match self.keycloak_client.get_user_by_email(email).await {
            Ok(user) => Ok(Some(user.try_into()?)),
            Err(keycloak_client::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn create_user(&self, command: CreateUserCommand) -> Result<User, CreateUserError> {
        let mut attributes = AttributeMap::default();
        if let Some(number) = command.phone_number {
            attributes.put("phoneNumber", number);
        }
        let request = CreateUserRequest {
            first_name: Some(command.first_name),
            last_name: Some(command.last_name),
            email: Some(command.email),
            email_verified: false,
            enabled: true,
            groups: vec![],
            attributes,
            credentials: vec![Credentials {
                credentials_type: crate::infrastructure::keycloak_client::CredentialsType::Password,
                value: command.password,
                temporary: false,
            }],
        };

        match self.keycloak_client.create_user(&request).await {
            Ok(response) => Ok(response.try_into()?),
            Err(keycloak_client::Error::Conflict) => Err(CreateUserError::UserAlreadyExists),
            Err(e) => Err(CreateUserError::TechnicalFailure(e.into())),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn update_user(
        &self,
        id: Uuid,
        command: UpdateUserCommand,
    ) -> Result<User, UpdateUserError> {
        let kc_user = self
            .keycloak_client
            .get_user_by_id(id)
            .await
            .map_err(|e| match e {
                keycloak_client::Error::NotFound => UpdateUserError::UserNotFound,
                e => UpdateUserError::TechnicalFailure(e.into()),
            })?;
        let email_verified = match &command.email {
            e @ Some(_) if &kc_user.email != e => Some(false),
            _ => None,
        };

        let mut attributes = kc_user.attributes;
        if let Some(number) = command.phone_number {
            attributes.put("phoneNumber", number);
        }

        let request = UpdateUserRequest {
            first_name: command.first_name.or(kc_user.first_name),
            last_name: command.last_name.or(kc_user.last_name),
            email: command.email.or(kc_user.email),
            email_verified,
            attributes: Some(attributes),
            credentials: command.password.map(|new_password| {
                vec![Credentials {
                    credentials_type: keycloak_client::CredentialsType::Password,
                    value: new_password,
                    temporary: false,
                }]
            }),
            ..Default::default()
        };

        match self.keycloak_client.update_user(id, &request).await {
            Ok(response) => Ok(response.try_into()?),
            Err(keycloak_client::Error::NotFound) => Err(UpdateUserError::UserNotFound),
            Err(e) => Err(UpdateUserError::TechnicalFailure(e.into())),
        }
    }
}

impl TryFrom<keycloak_client::UserItem> for User {
    type Error = anyhow::Error;

    fn try_from(value: keycloak_client::UserItem) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            first_name: value
                .first_name
                .with_context(|| "User without first name")?,
            last_name: value.last_name.with_context(|| "User without last name")?,
            email: value.email.with_context(|| "User without e-mail")?,
            phone_number: value
                .attributes
                .get("phoneNumber")
                .map(|str| str.to_string()),
        })
    }
}
