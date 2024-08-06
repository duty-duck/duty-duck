use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use uuid::Uuid;

use super::http_monitor::{HttpMonitorErrorKind, HttpMonitorStatus};

/// The base struct used by all incident types
#[derive(Serialize, Deserialize, TS, Debug, Clone, FromRow)]
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
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export)]
pub struct IncidentWithSources {
    #[serde(flatten)]
    pub incident: Incident,
    pub sources: HashSet<IncidentSource>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[serde(tag = "causeType", rename_all_fields = "camelCase")]
#[ts(export)]
pub enum IncidentCause {
    HttpMonitorIncidentCause {
        error_kind: HttpMonitorErrorKind,
        http_code: Option<i16>
    }
}

#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
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

/// An enum the can hold one of the different incident types at runtime
#[derive(Serialize, Deserialize, TS, Debug, Clone, PartialEq, Eq, Hash)]
#[ts(export)]
#[serde(tag = "type")]
pub enum IncidentSource {
    HttpMonitor { id: Uuid },
}
