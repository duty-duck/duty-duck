use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::http_monitor::HttpMonitorErrorKind;

/// The base struct used by all incident types
#[derive(Serialize, Deserialize, TS, Debug, Clone, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Incident {
    pub organization_id: Uuid,
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    #[sqlx(json)]
    pub cause: Option<IncidentCause>,
    pub status: IncidentStatus,
    pub priority: IncidentPriority,
    pub incident_source_type: IncidentSourceType,
    pub incident_source_id: Uuid,
    pub acknowledged_by: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, ToSchema)]
#[serde(tag = "causeType", rename_all_fields = "camelCase")]
#[ts(export)]
pub enum IncidentCause {
    HttpMonitorIncidentCause {
        error_kind: HttpMonitorErrorKind,
        http_code: Option<i16>,
    },
}

#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum IncidentStatus {
    Resolved = 0,
    Ongoing = 1,
}

impl From<i16> for IncidentStatus {
    fn from(value: i16) -> Self {
        match value {
            0 => Self::Resolved,
            1 => Self::Ongoing,
            _ => panic!("invalid IncidentStatus discriminant: {value}"),
        }
    }
}

impl IncidentStatus {
    pub const ALL: [Self; 2] = [Self::Resolved, Self::Ongoing];
}

#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum IncidentPriority {
    Emergency = 1,
    Critical = 2,
    Major = 3,
    Minor = 4,
    Warning = 5,
    Notice = 6,
}

impl From<i16> for IncidentPriority {
    fn from(value: i16) -> Self {
        match value {
            1 => Self::Emergency,
            2 => Self::Critical,
            3 => Self::Major,
            4 => Self::Minor,
            5 => Self::Warning,
            6 => Self::Notice,
            _ => panic!("invalid IncidentPriority discriminant: {value}"),
        }
    }
}

impl IncidentPriority {
    pub const ALL: [Self; 6] = [
        Self::Emergency,
        Self::Critical,
        Self::Major,
        Self::Minor,
        Self::Warning,
        Self::Notice,
    ];
}

#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum IncidentSourceType {
    HttpMonitor = 0,
}

impl From<i16> for IncidentSourceType {
    fn from(value: i16) -> Self {
        match value {
            0 => Self::HttpMonitor,
            _ => panic!("invalid IncidentSourceType discriminant: {value}"),
        }
    }
}

/// An enum the can hold one of the different incident types at runtime
#[derive(Serialize, Deserialize, TS, Debug, Clone, PartialEq, Eq, Hash)]
#[ts(export)]
#[serde(tag = "type")]
pub enum IncidentSource {
    HttpMonitor { id: Uuid },
}

#[derive(Debug, Clone)]
pub struct NewIncident {
    pub organization_id: Uuid,
    pub created_by: Option<Uuid>,
    pub status: IncidentStatus,
    pub priority: IncidentPriority,
    pub source: IncidentSource,
    pub cause: Option<IncidentCause>,
}
