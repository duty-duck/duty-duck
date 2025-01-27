use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::http_monitor::HttpMonitorErrorKind;

/// An event that is recorded for an incident.
#[derive(Serialize, Deserialize, TS, Debug, Clone, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IncidentEvent {
    pub organization_id: Uuid,
    pub incident_id: Uuid,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub event_type: IncidentEventType,
    #[sqlx(json)]
    pub event_payload: Option<IncidentEventPayload>,
}

/// A payload for an incident event
/// The nature of the payload is determined by the event type, and not all events have a payload
#[derive(Serialize, Deserialize, TS, Debug, Clone, ToSchema)]
#[serde(rename_all_fields = "camelCase")]
#[ts(export)]
pub enum IncidentEventPayload {
    /// This payload is bound to the event [IncidentEventType::Comment]
    Comment(CommentPayload),
    /// This payload is bound to the event [IncidentEventType::Notification]
    Notification(NotificationEventPayload),
    /// This payload is bound to the event [IncidentEventType::Acknowledged]
    Acknowledged(AcknowledgedEventPayload),
    /// This payload is bound to the event [IncidentEventType::MonitorPinged]
    MonitorPing(PingEventPayload),
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CommentPayload {
    editorjs_data: serde_json::Value,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AcknowledgedEventPayload {
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NotificationEventPayload {
    pub escalation_level: i16,
    pub sent_via_email: bool,
    pub sent_via_push_notification: bool,
    pub sent_via_sms: bool,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PingEventPayload {
    pub http_code: Option<i32>,
    pub error_kind: HttpMonitorErrorKind,
    pub http_headers: HashMap<String, String>,
    pub response_time_ms: u64,
    pub response_ip_address: Option<String>,
    pub resolved_ip_addresses: Vec<String>,
    pub response_file_id: Option<Uuid>,
    pub screenshot_file_id: Option<Uuid>,
}

#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum IncidentEventType {
    // Generic events for any incident

    /// This event is created when the incident is created
    Creation = 0,
    /// This event is created when notifications are sent for an incident (initial notification or escalation)
    Notification = 1,
    /// This event is created when the incident is resolved
    Resolution = 2,
    /// This event is created when a comment is added to an incident
    Comment = 3,
    /// This event is created when an incident is acknowledged by a user
    Acknowledged = 4,
    /// This event is created when an incident is confirmed (by a user or automatically)
    Confirmation = 5,

    // Monitor events (range 100-199)
    /// This event is created when a monitor is pinged
    MonitorPinged = 100,
    /// This event is created when a monitor is switched to recovering
    MonitorSwitchedToRecovering = 101,
    /// This event is created when a monitor is switched to suspicious
    MonitorSwitchedToSuspicious = 102,
    /// This event is created when a monitor is switched to down
    MonitorSwitchedToDown = 103,

    // Task events (range 200-299)
    /// This event is created when a scheduled task is switched to due
    TaskSwitchedToDue = 200,
    /// This event is created when a scheduled task is switched to late
    TaskSwitchedToLate = 201,
    /// This event is created when a scheduled task is switched to absent
    TaskSwitchedToAbsent = 202,

    /// This event is created on task-related incidents when a the task is started
    /// Although similar, it is distinct from [IncidentEventType::TaskRunStarted], which is created on task **run**-related incidents.
    /// We make a distinction between task-related incidents (a scheduled task is late, a scheduled task is absent) and task run-related incidents (a task run failed)
    TaskSwitchedToRunning = 203,

    // Task run events (range 300-399)
    /// This event is created when a task run is started
    TaskRunStarted = 300,
    /// This event is created when a task run is dead
    TaskRunIsDead = 301,
    /// This event is created when a task run is failed
    TaskRunFailed = 302,
}

impl From<i16> for IncidentEventType {
    fn from(value: i16) -> Self {
        match value {
            // generic events
            0 => Self::Creation,
            1 => Self::Notification,
            2 => Self::Resolution,
            3 => Self::Comment,
            4 => Self::Acknowledged,
            5 => Self::Confirmation,

            // monitor events
            100 => Self::MonitorPinged,
            101 => Self::MonitorSwitchedToRecovering,
            102 => Self::MonitorSwitchedToSuspicious,
            103 => Self::MonitorSwitchedToDown,

            // task events
            200 => Self::TaskSwitchedToDue,
            201 => Self::TaskSwitchedToLate,
            202 => Self::TaskSwitchedToAbsent,

            // task run events
            300 => Self::TaskRunStarted,
            301 => Self::TaskRunIsDead,
            302 => Self::TaskRunFailed,
            _ => panic!("invalid IncidentEventType discriminant: {value}"),
        }
    }
}
