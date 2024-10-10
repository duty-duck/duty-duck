use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use super::*;
use crate::domain::{
    entities::{http_monitor::*, incident::*, incident_event::*},
    use_cases::{incidents::*, shared::OrderDirection},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        incidents_router::list_incidents_handler,
        incidents_router::get_incident_handler,
        incidents_router::get_incident_timeline_handler
    ),
    components(schemas(
        ListIncidentsResponse,
        Incident,
        IncidentCause,
        IncidentStatus,
        IncidentPriority,
        IncidentSourceType,
        OrderIncidentsBy,
        HttpMonitor,
        HttpMonitorErrorKind,
        HttpMonitorStatus,
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
        TimelineItemUser
    ))
)]
struct ApiDoc;

pub fn redoc_router() -> Router<ApplicationState> {
    Router::new().merge(Redoc::with_url("/", ApiDoc::openapi()))
}
