use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use super::*;
use crate::domain::{
    entities::{entity_metadata::EntityMetadata, http_monitor::*, incident::*, incident_event::*, task::{Task, TaskId, TaskStatus}, task_run::{TaskRun, TaskRunStatus}, user::UserNameInfo},
    use_cases::{http_monitors::*, incidents::*, shared::OrderDirection, tasks::{CreateTaskCommand, GetTaskResponse, ListTasksResponse}},
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
        Task,
        TaskRun,
        ListTasksResponse,
        CreateTaskCommand,
        GetTaskResponse,
    ))
)]
struct ApiDoc;

pub fn redoc_router() -> Router<ApplicationState> {
    Router::new().merge(Redoc::with_url("/", ApiDoc::openapi()))
}
