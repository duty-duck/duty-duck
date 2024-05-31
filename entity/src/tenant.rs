use sea_orm::entity::prelude::*;
use sea_orm::TryFromU64;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TenantId(pub String);

/// The TryFromU64 trait is required for any type we intend to use as a primary key
impl TryFromU64 for TenantId {
    fn try_from_u64(n: u64) -> Result<Self, DbErr> {
        <String as TryFromU64>::try_from_u64(n).map(TenantId)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tenant")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: TenantId,
    pub name: String,
    pub stripe_customer_id: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::http_monitor::Entity")]
    HttpMonitor,
    #[sea_orm(has_many = "super::http_monitor::Entity")]
    UserAccount,
}

impl Related<super::user_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserAccount.def()
    }
}

impl Related<super::http_monitor::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HttpMonitor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
