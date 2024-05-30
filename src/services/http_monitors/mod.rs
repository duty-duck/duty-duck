use ::entity::http_monitor;
use anyhow::Result;
use chrono::Utc;
use sea_orm::prelude::*;
use sea_orm::*;
use sea_query::Expr;
use url::Url;

use crate::app_env::AppConfig;
use std::sync::Arc;

pub struct HttpMonitorsService {
    app_config: Arc<AppConfig>,
    db: DatabaseConnection,
}

pub struct CreateMonitorParams {
    pub owner_user_id: Uuid,
    pub interval_seconds: u64,
    pub url: Url,
}

pub struct GetMonitorParams {
    pub owner_user_id: Uuid,
    pub page: u64,
    pub items_per_page: u64,
}

impl HttpMonitorsService {
    pub fn new(app_config: Arc<AppConfig>, db: DatabaseConnection) -> Self {
        Self { app_config, db }
    }

    pub async fn list_monitors(
        &self,
        params: GetMonitorParams,
    ) -> anyhow::Result<Vec<http_monitor::Model>> {
        let items_per_page = params.items_per_page.min(100);
        let monitors = http_monitor::Entity::find()
            .filter(http_monitor::Column::OwnerUserAccount.eq(params.owner_user_id))
            .limit(items_per_page)
            .offset(items_per_page * params.page)
            .all(&self.db)
            .await?;
        Ok(monitors)
    }

    pub async fn create_monitor(&self, params: CreateMonitorParams) -> anyhow::Result<()> {
        let now = Utc::now();
        let new_monitor = http_monitor::ActiveModel {
            url: Set(params.url.to_string()),
            interval_seconds: Set(params.interval_seconds as i32),
            owner_user_account: Set(params.owner_user_id),
            created_at: Set(now),
            ..Default::default()
        };
        http_monitor::Entity::insert(new_monitor)
            .exec(&self.db)
            .await?;
        Ok(())
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
