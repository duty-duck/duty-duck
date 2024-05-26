use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserAccount::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserAccount::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(UserAccount::FullName).string().not_null())
                    .col(
                        ColumnDef::new(UserAccount::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(UserAccount::Password).string().not_null())
                    .col(ColumnDef::new(UserAccount::EmailConfirmedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(UserAccount::StripeCustomerId).string())
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
pub enum UserAccount {
    Table,
    Id,
    FullName,
    Email,
    EmailConfirmedAt,
    Password,
    StripeCustomerId,
    CreatedAt,
    UpdatedAt,
}