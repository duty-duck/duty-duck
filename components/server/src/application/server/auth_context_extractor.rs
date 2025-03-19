use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

use chrono::Utc;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{DecodingKey, Validation};
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

use crate::application::application_state::ApplicationState;
use crate::domain::entities::authorization::{
    ApiAccessToken, AuthContext, OriginalAuthenticationToken,
};
use crate::domain::entities::organization::OrganizationRoleSet;
use crate::domain::ports::api_access_token_repository::ApiAccessTokenRepository;
use crate::domain::ports::organization_repository::OrganizationRepository;
use crate::domain::use_cases::users::pick_or_create_default_org;

#[derive(Deserialize)]
struct Claims {
    active_organization: Option<ActiveOrganizationClaim>,
    sub: Uuid,
}

#[derive(Deserialize)]
struct ActiveOrganizationClaim {
    id: Option<Uuid>,
    #[serde(default)]
    role: Vec<String>,
}

impl FromRequestParts<ApplicationState> for AuthContext {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApplicationState,
    ) -> Result<Self, Self::Rejection> {
        let mut auth_context = match bearer_token_authentication(parts, state).await {
            Ok(auth_context) => auth_context,
            _ => api_token_authentication(parts, state).await?,
        };

        if auth_context.active_organization_id.is_none() {
            let (org_id, roles) = pick_or_create_default_org(
                &state.adapters.organization_repository,
                &state.adapters.user_repository,
                auth_context.active_user_id,
            )
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Missing active organization and could not create default organization for user"))?;

            auth_context.active_organization_id = Some(org_id);
            auth_context.active_organization_roles = roles;
        }

        Ok(auth_context)
    }
}

async fn bearer_token_authentication(
    parts: &mut Parts,
    state: &ApplicationState,
) -> Result<AuthContext, (StatusCode, &'static str)> {
    let authorization_header = parts
        .headers
        .get("authorization")
        .ok_or((StatusCode::UNAUTHORIZED, "Authorization header is missing"))?;
    let bearer_token = authorization_header
        .to_str()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid Authorization header"))?
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::BAD_REQUEST, "Invalid Authorization header"))?;
    let header = jsonwebtoken::decode_header(bearer_token)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid Authorization header"))?;
    let kid = header
        .kid
        .ok_or((StatusCode::BAD_REQUEST, "Invalid Authorization header"))?;
    let jwks = state.keycloak_client.get_jwks().await.map_err(|e| {
        error!(error = ?e, "Failed to retrieve JWKS from Keycloak");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve JWKS from Keycloak",
        )
    })?;
    let key = jwks.find(&kid).ok_or_else(|| {
        error!(kid = kid, "JWKS does not contain any key with this kid");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve JWKS from Keycloak",
        )
    })?;
    let key = DecodingKey::from_jwk(key).map_err(|e| {
        error!(error = ?e, "Cannot build decoding key from JWK");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to build decoding key from JWK",
        )
    })?;
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&state.access_token_audience);
    let token = jsonwebtoken::decode::<Claims>(bearer_token, &key, &validation).map_err(|e| {
        match e.kind() {
            ErrorKind::InvalidAudience =>  error!(error = ?e, "Failed to decode access token because of invalid audience. Verify the ACCESS_TOKEN_AUDIENCE configuration variable."),
            _ =>  error!(error = ?e, "Failed to decode access token. This should not happen, maybe scopes are missing in Keycloak ?")
        };

        (StatusCode::FORBIDDEN, "Failed to decode access token")
    })?;
    let auth_context = AuthContext {
        active_organization_id: token
            .claims
            .active_organization
            .as_ref()
            .and_then(|org| org.id),
        active_user_id: token.claims.sub,
        active_organization_roles: token
            .claims
            .active_organization
            .map(|org| org.role.into())
            .unwrap_or_default(),
        restricted_to_scopes: vec![],
        original_auth_token: Some(OriginalAuthenticationToken::BearerToken {
            bearer_token: bearer_token.to_string(),
        }),
    };

    Ok(auth_context)
}

async fn api_token_authentication(
    parts: &mut Parts,
    state: &ApplicationState,
) -> Result<AuthContext, (StatusCode, &'static str)> {
    let api_token_id = parts
        .headers
        .get("X-Api-Token-Id")
        .ok_or((StatusCode::UNAUTHORIZED, "X-Api-Token-Id header is missing"))?;

    let api_token_id = Uuid::parse_str(
        api_token_id
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid X-Api-Token-Id header"))?,
    )
    .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid X-Api-Token-Id header"))?;

    let api_token_secret_key = parts.headers.get("X-Api-Token-Secret-Key").ok_or((
        StatusCode::UNAUTHORIZED,
        "X-Api-Token-Secret-Key header is missing",
    ))?;

    let api_token_secret_key =
        ApiAccessToken::decode_secret_key(api_token_secret_key.to_str().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid X-Api-Token-Secret-Key header",
            )
        })?)
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid X-Api-Token-Secret-Key header",
            )
        })?;

    let access_token = state
        .adapters
        .api_token_repository
        .get_api_token(api_token_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve API token. Please retry later.",
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "API Token is invalid"))?;

    if access_token.expires_at <= Utc::now() {
        return Err((StatusCode::UNAUTHORIZED, "API Token is expired"));
    }

    if access_token.secret_key != api_token_secret_key {
        return Err((StatusCode::UNAUTHORIZED, "Invalid API Token secret key"));
    }

    let active_organization_roles = state
        .adapters
        .organization_repository
        .list_organization_roles_for_user(access_token.organization_id, access_token.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve user roles. Please retry later",
            )
        })?;

    let active_organization_roles = OrganizationRoleSet::from_roles(active_organization_roles);

    Ok(AuthContext {
        active_organization_id: Some(access_token.organization_id),
        active_user_id: access_token.user_id,
        active_organization_roles,
        restricted_to_scopes: access_token.scopes,
        original_auth_token: Some(OriginalAuthenticationToken::APITokenPair {
            id: api_token_id,
            secret_key: hex::encode(api_token_secret_key),
        }),
    })
}
