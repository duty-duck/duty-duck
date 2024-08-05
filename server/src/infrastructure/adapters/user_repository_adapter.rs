use std::sync::Arc;

use anyhow::Context;

use crate::domain::entities::user::*;
use crate::domain::ports::user_repository::UserRepository;
use crate::infrastructure::keycloak_client::{
    self, AttributeMap, CreateUserRequest, Credentials, KeycloakClient,
};

#[derive(Clone)]
pub struct UserRepositoryAdapter {
    pub keycloak_client: Arc<KeycloakClient>,
}

impl UserRepository for UserRepositoryAdapter {
    #[tracing::instrument(skip(self))]
    async fn create_user(&self, command: CreateUserCommand) -> Result<User, CreateUserError> {
        let request = CreateUserRequest {
            first_name: Some(command.first_name),
            last_name: Some(command.last_name),
            email: Some(command.email),
            email_verified: false,
            enabled: true,
            groups: vec![],
            attributes: AttributeMap::default(),
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
        })
    }
}
