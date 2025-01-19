use axum::response::IntoResponse;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use super::*;
use crate::domain::{
    entities::{
        entity_metadata::EntityMetadata,
        http_monitor::*,
        incident::*,
        incident_event::*,
        task::{BoundaryTask, TaskId, TaskStatus},
        task_run::{BoundaryTaskRun, TaskRunStatus},
        user::UserNameInfo,
    },
    use_cases::{
        http_monitors::*,
        incidents::*,
        shared::OrderDirection,
        tasks::{
            FinishTaskCommand, GetTaskResponse, ListTaskRunsResponse, ListTasksResponse, NewTask,
            StartTaskCommand,
        },
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        incidents_router::list_incidents_handler,
        incidents_router::get_incident_handler,
        incidents_router::get_incident_timeline_handler,
        http_monitors_router::get_http_monitor_handler,
        http_monitors_router::list_http_monitors_handler,
        http_monitors_router::create_http_monitor_handler,
        http_monitors_router::update_http_monitor_handler,
        http_monitors_router::archive_http_monitor_handler,
        http_monitors_router::toggle_http_monitor_handler,
        tasks_router::list_tasks_handler,
        tasks_router::create_task_handler,
        tasks_router::get_task_handler,
        tasks_router::start_task_handler,
        tasks_router::finish_task_handler,
        tasks_router::list_task_runs_handler,
        tasks_router::send_task_heartbeat_handler
    ),
    components(schemas(
        ListIncidentsResponse,
        Incident,
        IncidentWithUsers,
        IncidentCause,
        IncidentStatus,
        IncidentPriority,
        IncidentSourceType,
        OrderIncidentsBy,
        HttpMonitor,
        HttpMonitorErrorKind,
        HttpMonitorStatus,
        HttpMonitorIncidentCause,
        HttpMonitorIncidentCausePing,
        OrderDirection,
        IncidentEvent,
        IncidentEventPayload,
        CommentPayload,
        NotificationEventPayload,
        AcknowledgedEventPayload,
        IncidentEventType,
        GetIncidentResponse,
        GetIncidentTimelineResponse,
        TimelineItem,
        TimelineItemUser,
        EntityMetadata,
        UserNameInfo,
        UpdateHttpMonitorCommand,
        CreateHttpMonitorCommand,
        ListHttpMonitorsResponse,
        RequestHeaders,
        TaskId,
        TaskStatus,
        TaskRunStatus,
        BoundaryTask,
        BoundaryTaskRun,
        ListTasksResponse,
        GetTaskResponse,
        FinishTaskCommand,
        StartTaskCommand,
        ListTaskRunsResponse,
        NewTask,
    ))
)]
struct ApiDoc;

async fn openapi_handler() -> impl IntoResponse {
    Json(ApiDoc::openapi())
}

pub fn redoc_router() -> Router<ApplicationState> {
    Router::new()
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .route("/", get(openapi_handler))
}
