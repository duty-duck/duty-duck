use sea_orm_migration::prelude::*;

use crate::m20240522_094208_crate_auth_user_accounts::UserAccount;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HttpMonitor::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(HttpMonitor::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(HttpMonitor::Url).string().not_null())
                    .col(
                        ColumnDef::new(HttpMonitor::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(HttpMonitor::FirstPingAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(HttpMonitor::NextPingAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(HttpMonitor::LastPingAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(HttpMonitor::IntervalSeconds)
                            .unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(HttpMonitor::LastHttpCode).tiny_unsigned())
                    .col(ColumnDef::new(HttpMonitor::LastStatus).tiny_unsigned())
                    .col(ColumnDef::new(HttpMonitor::OwnerUserAccount).uuid())
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .from(HttpMonitor::Table, HttpMonitor::OwnerUserAccount)
                            .to(UserAccount::Table, UserAccount::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(HttpMonitor::Table)
                    .name("http_monitor_next_ping_at_idx")
                    .if_not_exists()
                    .col(HttpMonitor::NextPingAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HttpMonitor::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum HttpMonitor {
    Table,
    Id,
    Url,
    FirstPingAt,
    CreatedAt,
    LastPingAt,
    NextPingAt,
    IntervalSeconds,
    LastHttpCode,
    LastStatus,
    OwnerUserAccount,
}
