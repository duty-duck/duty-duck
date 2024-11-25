use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Uses by the push notification server to identify a device
#[derive(Debug, Clone, TS, Serialize, Deserialize, sqlx::Type)]
#[ts(export)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct PushNotificationToken(pub String);

/// A rapper arround an optional [PushNotificationToken], required to query the database using the `sqlx::query_as!` macro.
#[derive(Debug, Clone, TS, Serialize, Deserialize, sqlx::Type)]
#[ts(export)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct OptionalPushNotificationToken(pub Option<PushNotificationToken>);

impl From<Option<String>> for OptionalPushNotificationToken {
    fn from(value: Option<String>) -> Self {
        Self(value.map(PushNotificationToken))
    }
}

#[derive(Debug)]
pub struct PushNotification {
    pub title: String,
    pub body: String,
}
