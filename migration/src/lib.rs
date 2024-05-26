pub use sea_orm_migration::prelude::*;

mod m20240522_094208_crate_auth_user_accounts;
mod m20240526_191740_create_http_monitors;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240522_094208_crate_auth_user_accounts::Migration),
            Box::new(m20240526_191740_create_http_monitors::Migration),
        ]
    }
}
