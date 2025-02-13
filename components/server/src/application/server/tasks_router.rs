use crate::{
    application::application_state::{ApplicationState, ExtractAppState},
    domain::{
        entities::{
            authorization::AuthContext,
            task::{TaskError, TaskId},
        },
        use_cases::tasks::*,
    },
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::Query;
use tracing::warn;
use uuid::Uuid;

pub(crate) fn tasks_router() -> Router<ApplicationState> {
    Router::new()
        .route("/", get(list_tasks_handler).post(create_task_handler))
        .nest(
            "/{task_id}",
            Router::new()
                .route("/", get(get_task_handler).patch(update_task_handler))
                .route("/start", post(start_task_handler))
                .route("/archive", post(archive_task_handler))
                .route("/finish", post(finish_task_handler))
                .route("/heartbeat", post(send_task_heartbeat_handler))
                .route("/runs/{started_at}", get(get_task_run_handler))
                .route("/runs", get(list_task_runs_handler)),
        )
}

/// List all tasks for the current organization
#[utoipa::path(
    get,
    path = "/tasks",
    params(ListTasksParams),
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
    request_body(
        content = CreateTaskCommand,
        description = "The command to create a task",
        content_type = "application/json"
    ),
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
    Json(command): Json<CreateTaskCommand>,
) -> impl IntoResponse {
    match create_task_use_case(&auth_context, &app_state.adapters.task_repository, command).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(CreateTaskError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(CreateTaskError::TaskError(TaskError::InvalidCronSchedule { details })) => (
            StatusCode::BAD_REQUEST,
            format!("Invalid cron schedule: {details}"),
        )
            .into_response(),
        Err(CreateTaskError::TaskError(TaskError::InvalidScheduleTimezone { details })) => (
            StatusCode::BAD_REQUEST,
            format!("Invalid schedule timezone: {details}"),
        )
            .into_response(),
        Err(CreateTaskError::TaskAlreadyExists(task_id)) => (
            StatusCode::CONFLICT,
            format!("Task already exists: {task_id}"),
        )
            .into_response(),
        Err(e) => {
            warn!(error = ?e, "Technical failure occured while creating a task");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get a single task by its id. The provided id can be either the user-defined id (the `user-id` field) or the technical UUID (the `id` field)
/// Note that if you use the user-defined id, you may only retrieve tasks that are not archived. If want to retrieve archived tasks, use the UUID instead.
/// A user-defiend id can only be used for a single active task within your organization.
#[utoipa::path(
    get,
    path = "/tasks/:user_id",
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
    Path(user_id): Path<TaskId>,
) -> impl IntoResponse {
    match get_task(&auth_context, &app_state.adapters.task_repository, user_id).await {
        Ok(response) => Json(response).into_response(),
        Err(GetTaskError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(GetTaskError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(GetTaskError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting a task");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// List all runs for a task
///
/// This endpoint can be used to get a paginated list of task runs for a task.
/// The provided task id can be either the user-defined id (the `user-id` field) or the technical UUID (the `id` field).
#[utoipa::path(
    get,
    path = "/tasks/:user_id/runs",
    params(ListTaskRunsParams),
    responses(
        (status = 200, body = ListTaskRunsResponse),
        (status = 403, description = "User is not allowed to list task runs"),
        (status = 500, description = "Technical failure occured while listing task runs")
    )
)]
async fn list_task_runs_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    Path(task_id): Path<TaskId>,
    Query(params): Query<ListTaskRunsParams>,
) -> impl IntoResponse {
    match list_task_runs_use_case(
        &auth_context,
        &app_state.adapters.task_repository,
        &app_state.adapters.task_run_repository,
        task_id,
        params,
    )
    .await
    {
        Ok(response) => Json(response).into_response(),
        Err(ListTaskRunsError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(ListTaskRunsError::TaskNotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(ListTaskRunsError::TechnicalFailure(e)) => {
            warn!("Technical failure occured while listing task runs: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Start a new run for a task
///
/// This will mark the task as running and create a new task run. A task cannot be started if it is already running or if it is archived.
///
/// The provided task id can be either the user-defined id (the `user-id` field) or the technical UUID (the `id` field).
#[utoipa::path(
    post,
    path = "/tasks/:task_id/start",
    request_body(
        content = Option<StartTaskCommand>,
        description = "An optional command to start a task run",
        content_type = "application/json"
    ),
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
    Json(command): Json<Option<StartTaskCommand>>,
) -> impl IntoResponse {
    match start_task_use_case(
        &auth_context,
        StartTaskUseCaseOpts {
            task_repository: &app_state.adapters.task_repository,
            task_run_repository: &app_state.adapters.task_run_repository,
            incident_repository: &app_state.adapters.incident_repository,
            incident_event_repository: &app_state.adapters.incident_event_repository,
            incident_notification_repository: &app_state.adapters.incident_notification_repository,
            task_id,
            command,
        },
    )
    .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(StartTaskError::Forbidden) => (
            StatusCode::FORBIDDEN,
            "User is not allowed to start this task",
        )
            .into_response(),
        Err(StartTaskError::TaskNotFound) => {
            (StatusCode::NOT_FOUND, "Task not found").into_response()
        }
        Err(StartTaskError::TaskAlreadyStarted) => {
            (StatusCode::CONFLICT, "Task already started").into_response()
        }
        Err(StartTaskError::TaskIsArchived) => (
            StatusCode::CONFLICT,
            "Task has been archived and can no longer be updated.",
        )
            .into_response(),
        Err(StartTaskError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while starting a task");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Send a heartbeat for a running task, to indicate that it is still running
/// Without a regular heartbeat, a running task will eventually be considered failed and retried.
/// This endpoint can only be used on running tasks.
///
/// The provided task id can be either the user-defined id (the `user-id` field) or the technical UUID (the `id` field).
#[utoipa::path(
    post,
    path = "/tasks/:task_id/heartbeat",
    responses(
        (status = 200, description = "Heartbeat sent successfully"),
        (status = 403, description = "User is not authorized to send a heartbeat for this task"),
        (status = 404, description = "Task not found"),
        (status = 400, description = "Task is not running"),
        (status = 500, description = "Technical failure occured while sending a heartbeat")
    )
)]
async fn send_task_heartbeat_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    Path(task_id): Path<TaskId>,
) -> impl IntoResponse {
    match send_task_heartbeat_use_case(
        &auth_context,
        &app_state.adapters.task_repository,
        &app_state.adapters.task_run_repository,
        task_id,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(SendTaskHeartbeatError::Forbidden) => (
            StatusCode::FORBIDDEN,
            "User is not allowed to send a heartbeat for this task",
        )
            .into_response(),
        Err(SendTaskHeartbeatError::TaskNotFound) => {
            (StatusCode::NOT_FOUND, "Task not found").into_response()
        }
        Err(SendTaskHeartbeatError::TaskIsNotRunning) => {
            (StatusCode::BAD_REQUEST, "Task is not running").into_response()
        }
        Err(SendTaskHeartbeatError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while sending a heartbeat");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Finish a running task
///
/// This will mark the task as finished, and record the exit code and error message if the task failed.
/// The provided task id can be either the user-defined id (the `user-id` field) or the technical UUID (the `id` field).
#[utoipa::path(
    post,
    path = "/tasks/:task_id/finish",
    request_body(
        content = FinishTaskCommand,
        description = "The command to finish a task",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Task finished successfully"),
        (status = 403, description = "User is not authorized to finish a task"),
        (status = 404, description = "Task not found"),
    )
)]
async fn finish_task_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    Path(task_id): Path<TaskId>,
    Json(command): Json<FinishTaskCommand>,
) -> impl IntoResponse {
    match finish_task_use_case(
        &auth_context,
        &app_state.adapters.task_repository,
        &app_state.adapters.task_run_repository,
        &app_state.adapters.incident_repository,
        &app_state.adapters.incident_event_repository,
        &app_state.adapters.incident_notification_repository,
        task_id,
        command,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(FinishTaskError::Forbidden) => (
            StatusCode::FORBIDDEN,
            "User is not allowed to finish this task",
        )
            .into_response(),
        Err(FinishTaskError::NotFound) => (StatusCode::NOT_FOUND, "Task not found").into_response(),
        Err(FinishTaskError::TaskIsNotRunning) => {
            (StatusCode::BAD_REQUEST, "Task is not running").into_response()
        }
        Err(FinishTaskError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while finishing a task");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Get a single task run of a task
///
/// The provided task id can be either the user-defined id (the `user-id` field) or the technical UUID (the `id` field).
#[utoipa::path(
    get,
    path = "/tasks/:task_id/runs/:task_run_id",
    responses(
        (status = 200, body = GetTaskRunResponse),
        (status = 403, description = "User is not authorized to get a task run"),
        (status = 404, description = "Task run not found")
    )
)]
async fn get_task_run_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    Path((task_id, task_run_id)): Path<(TaskId, Uuid)>,
) -> impl IntoResponse {
    match get_task_run(
        &auth_context,
        &app_state.adapters.task_repository,
        &app_state.adapters.task_run_repository,
        task_id,
        task_run_id,
    )
    .await
    {
        Ok(response) => Json(response).into_response(),
        Err(GetTaskRunError::Forbidden) => StatusCode::FORBIDDEN.into_response(),
        Err(GetTaskRunError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(GetTaskRunError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while getting a task run");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Archive a task
///
/// After a task has been archived, it can no longer be ran or updated. The task cannot be archived while it is running.
/// Once the task is archived, the user-friendly of this task can be repurposed for another task.
#[utoipa::path(
    post,
    path = "/tasks/:task_id/archive",
    responses(
        (status = 200, description = "Task archived successfully"),
        (status = 403, description = "User is not authorized to archive a task"),
        (status = 404, description = "Task not found"),
        (status = 409, description = "Task is running or already-archived"),
        (status = 500, description = "Technical failure occured while starting a task")
    )
)]
async fn archive_task_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    Path(task_id): Path<TaskId>,
) -> impl IntoResponse {
    match archive_task(
        &app_state.adapters.task_repository,
        &app_state.adapters.task_run_repository,
        &auth_context,
        task_id,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(ArchiveTaskError::Forbidden) => (
            StatusCode::FORBIDDEN,
            "User is not allowed to archive this task",
        )
            .into_response(),
        Err(ArchiveTaskError::TaskNotFound) => {
            (StatusCode::NOT_FOUND, "Task not found").into_response()
        }
        Err(ArchiveTaskError::CannotArchiveRunningTask) => (
            StatusCode::CONFLICT,
            "Cannot archive a task while it is running",
        )
            .into_response(),
        Err(ArchiveTaskError::AlreadyArchived) => {
            (StatusCode::CONFLICT, "The task is already archived").into_response()
        }
        Err(ArchiveTaskError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while starting a task");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Update a task
#[utoipa::path(
    patch,
    path = "/tasks/:task_id",
    request_body(
        content = UpdateTaskCommand,
        description = "The command to update a task",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Task updated successfully"),
        (status = 403, description = "User is not authorized to update a task"),
        (status = 404, description = "Task not found"),
        (status = 409, description = "New task is already used"),
        (status = 500, description = "Technical failure occured while starting a task")
    )
)]
async fn update_task_handler(
    State(app_state): ExtractAppState,
    auth_context: AuthContext,
    Path(task_id): Path<TaskId>,
    Json(command): Json<UpdateTaskCommand>,
) -> impl IntoResponse {
    match update_task(
        &auth_context,
        &app_state.adapters.task_repository,
        &app_state.adapters.task_run_repository,
        task_id,
        command,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(UpdateTaskError::Forbidden) => (
            StatusCode::FORBIDDEN,
            "User is not allowed to update this task",
        )
            .into_response(),
        Err(UpdateTaskError::TaskNotFound) => {
            (StatusCode::NOT_FOUND, "Task not found").into_response()
        }
        Err(UpdateTaskError::UserIdConflict) => (
            StatusCode::CONFLICT,
            "Cannot update the task's id because the new id is already used by another task",
        )
            .into_response(),
        Err(UpdateTaskError::TaskError(e)) => {
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
        Err(UpdateTaskError::TaskArchived) => {
            (StatusCode::CONFLICT, "The task is archived").into_response()
        }
        Err(UpdateTaskError::TechnicalFailure(e)) => {
            warn!(error = ?e, "Technical failure occured while starting a task");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
