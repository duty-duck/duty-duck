use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Serialize, Deserialize, TS, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct HttpMonitor {
    pub organization_id: Uuid,
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub first_ping_at: Option<DateTime<Utc>>,
    pub next_ping_at: Option<DateTime<Utc>>,
    pub last_ping_at: Option<DateTime<Utc>>,
    pub interval_seconds: i64,
    pub last_http_code: Option<i16>,
    pub status: HttpMonitorStatus,
    pub status_counter: i16
}

impl HttpMonitor {
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_seconds as u64)
    }
}

#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
pub enum HttpMonitorStatus {
    Unknown = -1,
    Inactive = 0,
    Up = 1,
    Recovering = 2,
    Suspicious = 3,
    Down = 4,
}

impl From<i16> for HttpMonitorStatus {
    fn from(value: i16) -> Self {
        match value {
            -1 => Self::Unknown,
            0 => Self::Inactive,
            1 => Self::Up,
            2 => Self::Recovering,
            3 => Self::Suspicious,
            4 => Self::Down,
            _ => panic!("invalid HttpMonitorStatus discriminant: {value}")

        }
    }
}