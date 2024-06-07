use axum::{
    async_trait,
    extract::{rejection::HostRejection, FromRequestParts, Host},
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
};
use entity::tenant;

use crate::app_env::AppEnv;

pub struct CurrentTenant(pub tenant::Model);

#[async_trait]
impl FromRequestParts<AppEnv> for CurrentTenant {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppEnv,
    ) -> Result<Self, Self::Rejection> {
        let Host(host) = Host::from_request_parts(parts, state)
            .await
            .map_err(HostRejection::into_response)?;
        match state.tenants_service.get_tenant_by_host(&host).await {
            Ok(Some(tenant)) => Ok(CurrentTenant(tenant)),
            // TOOD: better error management here
            _ => Err(Redirect::to("/").into_response()),
        }
    }
}
