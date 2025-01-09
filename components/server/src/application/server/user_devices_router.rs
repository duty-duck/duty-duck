use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use tracing::warn;
use uuid::Uuid;

use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{
        entities::authorization::AuthContext,
        use_cases::user_devices::{
            self, ListUserDevicesError, RegisterUserDeviceCommand, RegisterUserDeviceError,
            RemoveUserDeviceError,
        },
    },
};

pub fn user_devices_router() -> Router<ApplicationState> {
    Router::new()
        .route(
            "/",
            get(list_user_devices_handler).post(register_user_device_handler),
        )
        .route("/{user_device_id}", delete(remove_user_device_handler))
}

async fn remove_user_device_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Path(user_device_id): Path<Uuid>,
) -> impl IntoResponse {
    match user_devices::remove_user_device(
        &auth_context,
        &app_state.adapters.user_devices_repository,
        user_device_id,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(RemoveUserDeviceError::DeviceNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(RemoveUserDeviceError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while removing a user device");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn register_user_device_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
    Json(command): Json<RegisterUserDeviceCommand>,
) -> impl IntoResponse {
    match user_devices::register_user_device(
        &auth_context,
        &app_state.adapters.user_devices_repository,
        command,
    )
    .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(RegisterUserDeviceError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while creating a user device");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn list_user_devices_handler(
    auth_context: AuthContext,
    State(app_state): ExtractAppState,
) -> impl IntoResponse {
    match user_devices::list_user_devices(
        &auth_context,
        &app_state.adapters.user_devices_repository,
    )
    .await
    {
        Ok(res) => Json(res).into_response(),
        Err(ListUserDevicesError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while listing user devices from the database");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
