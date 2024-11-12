use std::time::Duration;

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use url::Url;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::protos;

use super::entity_metadata::EntityMetadata;

#[derive(Serialize, Deserialize, TS, Debug, Clone, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HttpMonitor {
    pub organization_id: Uuid,
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub first_ping_at: Option<DateTime<Utc>>,
    pub next_ping_at: Option<DateTime<Utc>>,
    pub last_ping_at: Option<DateTime<Utc>>,
    pub last_status_change_at: DateTime<Utc>,
    #[ts(type = "number")]
    pub recovery_confirmation_threshold: i16,
    #[ts(type = "number")]
    pub downtime_confirmation_threshold: i16,
    #[ts(type = "number")]
    pub interval_seconds: i64,
    pub last_http_code: Option<i16>,
    pub status: HttpMonitorStatus,
    pub status_counter: i16,
    pub error_kind: HttpMonitorErrorKind,
    #[sqlx(json)]
    pub metadata: EntityMetadata,
    pub email_notification_enabled: bool,
    pub push_notification_enabled: bool,
    pub sms_notification_enabled: bool,
}

impl HttpMonitor {
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_seconds as u64)
    }

    pub fn url(&self) -> anyhow::Result<Url> {
        Url::parse(&self.url).context("invalid url for monitor")
    }
}

#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum HttpMonitorStatus {
    Unknown = -1,
    Inactive = 0,
    Up = 1,
    Recovering = 2,
    Suspicious = 3,
    Down = 4,
}

impl HttpMonitorStatus {
    pub const ALL: [Self; 6] = [
        Self::Unknown,
        Self::Inactive,
        Self::Up,
        Self::Recovering,
        Self::Suspicious,
        Self::Down,
    ];
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
            _ => panic!("invalid HttpMonitorStatus discriminant: {value}"),
        }
    }
}

#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, ToSchema, Default)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum HttpMonitorErrorKind {
    Unknown = -1,
    #[default]
    None = 0,
    HttpCode = 1,
    Connect = 2,
    Builder = 3,
    Request = 4,
    Redirect = 5,
    Body = 6,
    Decode = 7,
    Timeout = 8,
    BrowserServiceCallFailed = 9,
}

impl From<protos::HttpErrorKind> for HttpMonitorErrorKind {
    fn from(value: protos::HttpErrorKind) -> Self {
        match value {
            protos::HttpErrorKind::Unknown => Self::Unknown,
            protos::HttpErrorKind::HttpCode => Self::HttpCode,
            protos::HttpErrorKind::Connect => Self::Connect,
            protos::HttpErrorKind::Builder => Self::Builder,
            protos::HttpErrorKind::Request => Self::Request,
            protos::HttpErrorKind::Redirect => Self::Redirect,
            protos::HttpErrorKind::Body => Self::Body,
            protos::HttpErrorKind::Decode => Self::Decode,
            protos::HttpErrorKind::Timeout => Self::Timeout,
        }
    }
}

impl From<i16> for HttpMonitorErrorKind {
    fn from(value: i16) -> Self {
        match value {
            -1 => Self::Unknown,
            0 => Self::None,
            1 => Self::HttpCode,
            2 => Self::Connect,
            3 => Self::Builder,
            4 => Self::Request,
            5 => Self::Redirect,
            6 => Self::Body,
            7 => Self::Decode,
            8 => Self::Timeout,
            9 => Self::BrowserServiceCallFailed,
            _ => panic!("invalid HttpMonitorErrorKind discriminant: {value}"),
        }
    }
}
