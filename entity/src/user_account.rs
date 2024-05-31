use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub tenant_id: super::tenant::TenantId,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub full_name: String,
    pub role: Role,
    pub email: String,
    pub email_confirmed_at: Option<DateTimeUtc>,
    pub password: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(EnumIter, DeriveActiveEnum, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Role {
    Manager = 0,
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
