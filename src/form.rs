use std::ops::{Deref, DerefMut};

use axum::{
    async_trait,
    extract::{rejection::FormRejection, FromRequest, Request},
    response::IntoResponse,
    Form, RequestExt,
};
use serde::{de::DeserializeOwned, Deserialize};

use crate::{
    app_env::AppEnv,
    session::{CSRFToken, Session},
};

/// A [SecureForm] worksmostly like a [axum::Form], except that it can only be extracted from the request if and only if
/// it contains a "csrf_token" field that matches the "crst_token" field contained in the user's session
#[derive(Deserialize)]
pub struct SecureForm<T> {
    csrf_token: CSRFToken,
    #[serde(flatten)]
    pub payload: T,
}

impl<T> Deref for SecureForm<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl<T> DerefMut for SecureForm<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload
    }
}

pub enum SecureFormRejection {
    FormRejection(FormRejection),
    InvalidCSRFToken,
}

impl IntoResponse for SecureFormRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::FormRejection(e) => e.into_response(),
            // TODO: improve this
            Self::InvalidCSRFToken => "Invalid CRSF Token".into_response(),
        }
    }
}

#[async_trait]
impl<T> FromRequest<std::sync::Arc<AppEnv>> for SecureForm<T>
where
    T: DeserializeOwned + 'static,
{
    type Rejection = SecureFormRejection;

    async fn from_request(
        mut req: Request,
        state: &std::sync::Arc<AppEnv>,
    ) -> Result<Self, Self::Rejection> {
        let session = req
            .extract_parts_with_state::<Session, _>(state)
            .await
            .map_err(|_| SecureFormRejection::InvalidCSRFToken)?;
        let form = req
            .extract::<Form<SecureForm<T>>, _>()
            .await
            .map_err(SecureFormRejection::FormRejection)?;

        if session.csrf_token != form.csrf_token {
            return Err(SecureFormRejection::InvalidCSRFToken);
        }

        Ok(form.0)
    }
}
