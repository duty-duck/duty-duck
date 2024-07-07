use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

use crate::application::application_state::ApplicationState;
use crate::domain::entities::authorization::AuthContext;

#[async_trait]
impl FromRequestParts<ApplicationState> for AuthContext {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApplicationState,
    ) -> Result<Self, Self::Rejection> {
        todo!()
    }
}
