use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{
        entities::{authorization::AuthContext, task::TaskId},
        use_cases::tasks::{
          get_task, list_tasks,
            GetTaskError, ListTasksError, ListTasksParams,
        },
    },
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tracing::warn;

pub(crate) fn tasks_router() -> Router<ApplicationState> {
    Router::new()
        .route("/", get(list_tasks_handler).post(create_task_handler))
        .nest(
            "/:task_id",
            Router::new()
                .route("/", get(get_task_handler))
                .route("/start", post(start_task_handler))
                .route("/runs", get(list_task_runs_handler)),
        )
}

/// List all tasks for the current organization
#[utoipa::path(
    get,
    path = "/tasks",
    responses(
        (status = 200, body = ListTasksResponse),
        (status = 403, description = "User is not authorized to list tasks"),
        (status = 500, description = "Technical failure occured while listing tasks")
    )
)]
async fn list_tasks_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    Query(params): Query<ListTasksParams>,
) -> impl IntoResponse {
    match list_tasks(&auth_context, &app_state.adapters.task_repository, params).await {
        Ok(response) => Json(response).into_response(),
        Err(ListTasksError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ListTasksError::TechnicalFailure(e)) => {
            warn!("Technical failure occured while listing tasks: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Create a new task
///
/// A task is an external process that can be excuted manually or on a schedule, and needs to be monitored.
/// Once a task is registered, you can create task runs to track individual executions.
/// The platform will take care of identifying scheduled tasks that are running late or failing.
#[utoipa::path(
    post,
    path = "/tasks",
    responses(
        (status = 201, description = "Task created successfully"),
        (status = 403, description = "User is not authorized to create a task"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Technical failure occured while creating a task")
    )
)]
async fn create_task_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    // Json(command): Json<CreateTaskCommand>,
) -> impl IntoResponse {
    todo!()
}

/// Get a task by id
#[utoipa::path(
    get,
    path = "/tasks/:task_id",
    responses(
        (status = 200, body = GetTaskResponse),
        (status = 403, description = "User is not authorized to get a task"),
        (status = 404, description = "Task not found"),
        (status = 400, description = "Invalid task id"),
        (status = 500, description = "Technical failure occured while getting a task")
    )
)]
async fn get_task_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    Path(task_id): Path<TaskId>,
) -> impl IntoResponse {
    match get_task(&auth_context, &app_state.adapters.task_repository, task_id).await {
        Ok(response) => Json(response).into_response(),
        Err(GetTaskError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(GetTaskError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(GetTaskError::TechnicalFailure(e)) => {
            warn!("Technical failure occured while getting a task: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// List all runs for a task
#[utoipa::path(get, path = "/tasks/:task_id/runs")]
async fn list_task_runs_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    Path(task_id): Path<TaskId>,
) -> impl IntoResponse {
    todo!()
}

/// Start a new run for a task
#[utoipa::path(
    post,
    path = "/tasks/:task_id/start",
    responses(
        (status = 201, description = "Task run started successfully"),
        (status = 403, description = "User is not authorized to start a task"),
        (status = 404, description = "Task not found"),
        (status = 409, description = "Task already running"),
        (status = 500, description = "Technical failure occured while starting a task")
    )
)]
async fn start_task_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    Path(task_id): Path<TaskId>,
) -> impl IntoResponse {
    todo!()
}
