pub use sea_orm_migration::prelude::*;

mod m20240522_094208_crate_auth_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240522_094208_crate_auth_tables::Migration),
        ]
    }
}
