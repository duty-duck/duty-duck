use serde::{Serialize, Deserialize};
use sqlx::prelude::FromRow;
use ts_rs::TS;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::push_notification::OptionalPushNotificationToken;

#[derive(Debug, Clone, Serialize, TS, FromRow)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserDevice {
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub label: String,
    pub push_notification_token: OptionalPushNotificationToken,
    pub device_type: UserDeviceType
}

#[derive(sqlx::Type, Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum UserDeviceType {
    Unknown = -1,
    Desktop = 0,
    Mobile = 1,
}

impl UserDeviceType {
    #[allow(unused)]
    pub const ALL: [Self; 3] = [
        Self::Unknown,
        Self::Desktop,
        Self::Mobile,
    ];
}

impl From<i16> for UserDeviceType {
    fn from(value: i16) -> Self {
        match value {
            -1 => Self::Unknown,
            0 => Self::Desktop,
            1 => Self::Mobile,
            _ => panic!("invalid UserDeviceType discriminant: {value}"),
        }
    }
}
