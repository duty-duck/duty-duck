use ::entity::http_monitor;
use anyhow::Result;
use sea_orm::prelude::*;
use sea_orm::*;
use sea_query::Expr;

use crate::app_env::AppConfig;
use std::sync::Arc;

pub struct HttpMonitorsService {
    app_config: Arc<AppConfig>,
    db: DatabaseConnection,
}

impl HttpMonitorsService {
    pub fn new(app_config: Arc<AppConfig>, db: DatabaseConnection) -> Self {
        Self { app_config, db }
    }

    /// Returns a list of active http monitors that are due for a ping, i.e. monitors whose next_ping_at is in the past.
    ///
    /// This initiates a database transaction that locks the selected monitors and ignores any locked monitor,
    /// so this function never returns monitors that have already been locked by a concurrent transaction.
    /// The lock is released when [LockedHttpMonitors] goes out of scope, or when the [LockedHttpMonitors::transaction] is explicitly released
    pub async fn select_pendings_monitors(&self, limit: u64) -> Result<LockedHttpMonitors> {
        let transaction = self.db.begin().await?;

        let monitors = http_monitor::Entity::find()
            .lock_with_behavior(
                migration::LockType::Update,
                migration::LockBehavior::SkipLocked,
            )
            .filter(http_monitor::Column::NextPingAt.is_not_null())
            .filter(Expr::col(http_monitor::Column::NextPingAt).gte(Expr::current_timestamp()))
            .limit(limit)
            .all(&transaction)
            .await?;

        Ok(LockedHttpMonitors {
            monitors,
            transaction,
        })
    }
}

pub struct LockedHttpMonitors {
    /// The database transaction currently locking these monitors
    pub transaction: DatabaseTransaction,
    pub monitors: Vec<http_monitor::Model>,
}
