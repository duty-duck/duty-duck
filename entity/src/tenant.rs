use std::fmt::Display;

use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{ArrayType, ValueTypeErr};
use sea_orm::TryGetable;
use sea_orm::{sea_query::ValueType, TryFromU64};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TenantId(pub Uuid);

impl Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<TenantId> for Value {
    fn from(value: TenantId) -> Self {
        Value::String(Some(Box::new(value.0.to_string())))
    }
}

impl ValueType for TenantId {
    fn try_from(v: Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            Value::Uuid(Some(x)) => Ok(TenantId(*x)),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "TenantId".into()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::String(None)
    }
}

impl TryGetable for TenantId {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &QueryResult,
        index: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        <Uuid as TryGetable>::try_get_by(res, index).map(TenantId)
    }
}

#[derive(Debug)]
pub enum TenantError {}

/// The TryFromU64 trait is required for any type we intend to use as a primary key
impl TryFromU64 for TenantId {
    fn try_from_u64(n: u64) -> Result<Self, DbErr> {
        <Uuid as TryFromU64>::try_from_u64(n).map(TenantId)
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
    #[sea_orm(has_many = "super::user_account::Entity")]
    UserAccount,
    #[sea_orm(has_many = "super::subdomain::Entity")]
    Subdomain,
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

impl Related<super::subdomain::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subdomain.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
