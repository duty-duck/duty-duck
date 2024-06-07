use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::tenant::TenantId;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "subdomain")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub subdomain: String,
    pub tenant_id: TenantId,
    pub role: Role,
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

#[derive(EnumIter, DeriveActiveEnum, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Role {
    TenantPrincipalSubdomain = 0,
}
