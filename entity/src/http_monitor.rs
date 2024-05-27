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
    // TODO: how to deal with this column ?
    // pub interval: Duration,
    pub last_http_code: Option<i32>
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
