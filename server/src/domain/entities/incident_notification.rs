use serde::{Deserialize, Serialize};
use sqlx::*;
use uuid::Uuid;

use super::incident::IncidentCause;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct IncidentNotification {
    pub organization_id: Uuid,
    pub incident_id: Uuid,
    pub escalation_level: i16,
    pub notification_type: IncidentNotificationType,
    #[sqlx(json)]
    pub notification_payload: IncidentNotificationPayload,
    pub send_sms: bool,
    pub send_push_notification: bool,
    pub send_email: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IncidentNotificationPayload {
    pub incident_cause: IncidentCause,
    pub incident_http_monitor_url: Option<String>
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
pub enum IncidentNotificationType {
    IncidentCreation = 0,
    IncidentResolution = 1,
}

impl From<i16> for IncidentNotificationType {
    fn from(value: i16) -> Self {
        match value {
            0 => Self::IncidentCreation,
            1 => Self::IncidentResolution,
            _ => panic!("invalid IncidentNotificationType discriminant: {value}"),
        }
    }
}
