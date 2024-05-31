use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tenant::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tenant::Id).text().not_null().primary_key())
                    .col(ColumnDef::new(Tenant::Name).text().not_null())
                    .col(
                        ColumnDef::new(Tenant::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Tenant::StripeCustomerId).string())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserAccount::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserAccount::TenantId).text().not_null())
                    .col(
                        ColumnDef::new(UserAccount::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(UserAccount::Role).small_integer().not_null())
                    .col(ColumnDef::new(UserAccount::FullName).string().not_null())
                    .col(ColumnDef::new(UserAccount::Email).string().not_null())
                    .col(ColumnDef::new(UserAccount::Password).string().not_null())
                    .col(ColumnDef::new(UserAccount::EmailConfirmedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(UserAccount::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserAccount::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    // Use a composite primary key for user accounts
                    .primary_key(Index::create().col(UserAccount::TenantId).col(UserAccount::Id))
                    // Add a foreign key referencing the tenant
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserAccount::Table, UserAccount::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // Add a unique constraint to the e-mail
                    .index(
                        Index::create()
                            .col(UserAccount::TenantId)
                            .col(UserAccount::Email)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserAccount::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Tenant {
    Table,
    Id,
    Name,
    CreatedAt,
    StripeCustomerId,
}

#[derive(DeriveIden)]
pub enum UserAccount {
    Table,
    Id,
    TenantId,
    Role,
    FullName,
    Email,
    EmailConfirmedAt,
    Password,
    CreatedAt,
    UpdatedAt,
}
