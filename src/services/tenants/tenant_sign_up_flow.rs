use anyhow::{anyhow, Context};
use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use askama::Template;
use chrono::Utc;
use email_address::EmailAddress;
use entity::{subdomain, tenant, user_account};
use sea_orm::{EntityTrait, Set, TransactionTrait};
use thiserror::Error;
use url::Url;

use crate::{mailer::Mailer, services::auth::email_confirmation::EmailConfirmationToken};

use super::TenantsService;

#[derive(Template)]
#[template(path = "emails/signup-confirmation.html")]
struct SignupConfirmationEmail {
    email_confirmation_url: Url,
}

/// Params used to create a new tenant
pub struct SignUpParams {
    pub tenant_subdomain: String,
    pub tenant_name: String,
    pub manager_full_name: String,
    pub manager_email: EmailAddress,
    pub manager_password: String,
}

#[derive(Error, Debug)]
pub enum SignUpError {
    #[error("Tenant already exists")]
    TenantAlreadyExists,
    #[error(transparent)]
    TechnicalError(#[from] anyhow::Error),
}

impl TenantsService {
    pub async fn sign_up_tenant(&self, params: SignUpParams) -> Result<(), SignUpError> {
        // Get tenant by subdomain
        let existing_tenant = self
            .get_tenant_by_subdomain(&params.tenant_subdomain)
            .await?;

        if existing_tenant.is_some() {
            return Err(SignUpError::TenantAlreadyExists);
        }

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .argon
            .hash_password(params.manager_password.as_bytes(), &salt)
            .map_err(|_| SignUpError::TechnicalError(anyhow!("Failed to hash password")))?;

        let now = Utc::now();

        let tx = self
            .db
            .begin()
            .await
            .with_context(|| "Failed to begin a transaction")?;

        let tenant = tenant::ActiveModel {
            name: Set(params.tenant_name.clone()),
            created_at: Set(now),
            ..Default::default()
        };
        let tenant_id = tenant::Entity::insert(tenant)
            .exec(&tx)
            .await
            .map_err(|e| SignUpError::TechnicalError(anyhow!(e)))?
            .last_insert_id;

        let subdomain = subdomain::ActiveModel {
            tenant_id: Set(tenant_id),
            subdomain: Set(params.tenant_name),
            role: Set(subdomain::Role::TenantPrincipalSubdomain),
        };
        let user = user_account::ActiveModel {
            tenant_id: Set(tenant_id),
            role: Set(user_account::Role::Manager),
            full_name: Set(params.manager_full_name),
            password: Set(password_hash.to_string()),
            email: Set(params.manager_email.to_string()),
            updated_at: Set(now),
            created_at: Set(now),
            ..Default::default()
        };

        subdomain::Entity::insert(subdomain)
            .exec(&tx)
            .await
            .with_context(|| "Failed to insert subdomain")?;
        let (_, user_id) = user_account::Entity::insert(user)
            .exec(&tx)
            .await
            .with_context(|| "Failed to insert user account")?
            .last_insert_id;

        let user = user_account::Entity::find_by_id((tenant_id, user_id))
            .one(&tx)
            .await
            .with_context(|| "Failed to select user")?
            .unwrap();

        tx.commit()
            .await
            .with_context(|| "Failed to commit transaction")?;

        let confirmation_token = EmailConfirmationToken::build(
            tenant_id,
            &self.app_config.paseto_key,
            user.id,
            &params.manager_email,
        )
        .with_context(|| "Failed to build e-mail confirmation token")?;

        self.send_confirmation_email(&user, &confirmation_token)
            .await
            .with_context(|| "Failed to send confirmation e-mail")?;

        Ok(())
    }

    async fn send_confirmation_email(
        &self,
        user: &user_account::Model,
        confirmation_token: &EmailConfirmationToken,
    ) -> anyhow::Result<()> {
        let body = SignupConfirmationEmail {
            email_confirmation_url: EmailConfirmationToken::url(
                &self.app_config,
                confirmation_token,
            ),
        }
        .render()?;
        let message = Mailer::builder()
            .subject("Confirm your Duty Duck registration")
            .to(format!("{} <{}>", user.full_name, user.email).parse()?)
            .body(body)?;
        self.mailer.send(message).await
    }
}
