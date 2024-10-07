use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Serialize, Deserialize, TS, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IncidentEvent {
    pub organization_id: Uuid,
    pub incident_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub event_type: IncidentEventType,
    #[sqlx(json)]
    pub event_payload: Option<IncidentEventPayload>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[serde(rename_all_fields = "camelCase")]
#[ts(export)]
pub enum IncidentEventPayload {
    Comment(CommentPayload),
    Notification(NotificationEventPayload),
    Acknowledged(AcknowledgedEventPayload),
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CommentPayload {
    editorjs_data: serde_json::Value 
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AcknowledgedEventPayload {
    pub user_id: Uuid,
}


#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NotificationEventPayload {
    pub escalation_level: i16,
    pub sent_via_email: bool,
    pub sent_via_push_notification: bool,
    pub sent_via_sms: bool,
}

#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum IncidentEventType {
    Creation = 0,
    Notification = 1,
    Resolution = 2,
    Comment = 3,
    Acknowledged = 4,
}

impl From<i16> for IncidentEventType {
    fn from(value: i16) -> Self {
        match value {
            0 => Self::Creation,
            1 => Self::Notification,
            2 => Self::Resolution,
            3 => Self::Comment,
            _ => panic!("invalid IncidentEventType discriminant: {value}"),
        }
    }
}