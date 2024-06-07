use std::time::Duration;

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::tenant::TenantId;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "http_monitor")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub tenant_id: TenantId,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub url: String,
    pub created_at: DateTimeUtc,
    pub first_ping_at: Option<DateTimeUtc>,
    pub next_ping_at: Option<DateTimeUtc>,
    pub last_ping_at: Option<DateTimeUtc>,
    pub interval_seconds: i32,
    pub last_http_code: Option<i16>,
    pub last_status: Option<i16>,
}

impl Model {
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_seconds as u64)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id"
    )]
    Tenant,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
