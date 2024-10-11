use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{DecodingKey, Validation};
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

use crate::application::application_state::ApplicationState;
use crate::domain::entities::authorization::AuthContext;

#[derive(Deserialize)]
struct Claims {
    active_organization: ActiveOrganizationClaim,
    sub: Uuid,
    #[serde(rename = "lastName")]
    last_name: Option<String>,
    #[serde(rename = "firstName")]
    first_name: Option<String>,
    #[serde(rename = "phoneNumber")]
    #[allow(unused)]
    phone_number: Option<String>,
}

#[derive(Deserialize)]
struct ActiveOrganizationClaim {
    id: Uuid,
    role: Vec<String>,
}

#[async_trait]
impl FromRequestParts<ApplicationState> for AuthContext {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApplicationState,
    ) -> Result<Self, Self::Rejection> {
        let authorization_header = parts
            .headers
            .get("authorization")
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let token = authorization_header
            .to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::BAD_REQUEST)?;
        let header = jsonwebtoken::decode_header(token).map_err(|_| StatusCode::BAD_REQUEST)?;
        let kid = header.kid.ok_or(StatusCode::BAD_REQUEST)?;
        let jwks = state.keycloak_client.get_jwks().await.map_err(|e| {
            error!(error = ?e, "Failed to retrieve JWKS from Keycloak");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let key = jwks.find(&kid).ok_or_else(|| {
            error!(kid = kid, "JWKS does not contain any key with this kid");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let key = DecodingKey::from_jwk(key).map_err(|e| {
            error!(error = ?e, "Cannot build decoding key from JWK");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&[&state.access_token_audience]);
        let token = jsonwebtoken::decode::<Claims>(token, &key, &validation).map_err(|e| {
            match e.kind() {
                ErrorKind::InvalidAudience =>  error!(error = ?e, "Failed to decode access token because of invalid audience. Verify the ACCESS_TOKEN_AUDIENCE configuration variable."),
                _ =>  error!(error = ?e, "Failed to decode access token. This should not happen, maybe scopes are missing in Keycloak ?")
            };
            StatusCode::FORBIDDEN
        })?;
        let auth_context = AuthContext {
            active_organization_id: token.claims.active_organization.id,
            active_user_id: token.claims.sub,
            active_organization_roles: token.claims.active_organization.role.into(),
            first_name: token.claims.first_name,
            last_name: token.claims.last_name,
            // TODO: populate this field from the api access token when available
            restricted_to_scopes: vec![],
        };

        Ok(auth_context)
    }
}
