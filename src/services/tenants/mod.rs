pub mod tenant_sign_up_flow;

use std::sync::Arc;

use argon2::Argon2;
use entity::subdomain;
use entity::tenant;
use moka::future::Cache;
use sea_orm::prelude::*;
use sea_orm::QuerySelect;

use crate::app_env::AppConfig;
use crate::mailer::Mailer;

pub struct TenantsService {
    app_config: Arc<AppConfig>,
    db: DatabaseConnection,
    // TODO: implement cache eviction once we allow tenants to be edited
    tenant_per_subdomain_cache: Cache<String, tenant::Model>,
    argon: Argon2<'static>,
    mailer: Mailer,
}

impl TenantsService {
    pub fn new(app_config: Arc<AppConfig>, db: DatabaseConnection, mailer: Mailer) -> Self {
        Self {
            app_config,
            db,
            tenant_per_subdomain_cache: Cache::new(1000),
            argon: Argon2::default(),
            mailer,
        }
    }

    pub async fn get_tenant_by_subdomain(&self, subdomain: &str) -> anyhow::Result<Option<tenant::Model>> {
        // Attempt to retrieve the tenant from the cache
        if let Some(tenant) = self.tenant_per_subdomain_cache.get(subdomain).await {
            return Ok(Some(tenant));
        }

        let tenant_opt: Option<tenant::Model> = 
        // Find the main subdomain for this tenant
        subdomain::Entity::find_by_id(subdomain)
            .filter(subdomain::Column::Role.eq(subdomain::Role::TenantPrincipalSubdomain))
            .filter(subdomain::Column::Subdomain.eq(subdomain))
            .select_only()
            // Join the tenant table to retrieve the tenant's info
            .inner_join(tenant::Entity)
            .into_model()
            // Get the first result
            .one(&self.db)
            .await?;

        // Put the tenant in the cache
        if let Some(tenant) = &tenant_opt {
            self.tenant_per_subdomain_cache
                .insert(subdomain.to_string(), tenant.clone())
                .await;
        }

        Ok(tenant_opt)
    }

    pub async fn get_tenant_by_host(
        &self,
        request_host: &str,
    ) -> anyhow::Result<Option<tenant::Model>> {
        let subdomain = match request_host.strip_suffix(&self.app_config.domain) {
            Some(d) => d,
            None => return Ok(None),
        };

        self.get_tenant_by_subdomain(subdomain).await
    }
}
