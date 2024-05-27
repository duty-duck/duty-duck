use std::time::Duration;

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "http_monitor")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: Uuid,
    pub url: String,
    pub created_at: DateTimeUtc,
    pub first_ping_at: Option<DateTimeUtc>,
    pub next_ping_at: Option<DateTimeUtc>,
    pub last_ping_at: Option<DateTimeUtc>,
    pub interval_seconds: u64,
    pub last_http_code: Option<u16>,
    pub last_status: Option<u16>,
    pub owner_user_account: Uuid
}

impl Model {
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_seconds)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user_account::Entity",
        from = "Column::OwnerUserAccount",
        to = "super::user_account::Column::Id"
    )]
    UserAccount
}

impl Related<super::user_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAccount.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
