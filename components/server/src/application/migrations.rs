use anyhow::Context;
use clap::Subcommand;
use sqlx::postgres::PgPoolOptions;

#[derive(Subcommand, Debug)]
pub enum MigrationsCommand {
    /// Run any pending migrations against the database; and, validate previously applied migrations
    /// against the current migration source to detect accidental changes in previously-applied migrations.
    Run,
    /// Run down migrations against the database until a specific version.
    Undo { target: i64 },
}

pub async fn run_migrations(command: MigrationsCommand) -> anyhow::Result<()> {
    let migrator = sqlx::migrate!("./migrations");
    let database_url = std::env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .with_context(|| "Failed to connect to the database")?;

    match command {
        MigrationsCommand::Run => {
            migrator.run(&pool).await?;
            tracing::info!("Migrations applied successfully");
        }
        MigrationsCommand::Undo { target } => {
            migrator.undo(&pool, target).await?;
            tracing::info!("Migrations undone successfully");
        }
    }

    Ok(())
}
