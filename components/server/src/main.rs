#[macro_use]
extern crate rust_i18n;

use application::migrations::MigrationsCommand;
use clap::*;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod application;
mod domain;
mod infrastructure;
mod protos;
mod shared;

// Initialize i18n
rust_i18n::i18n!("locales", fallback = "en");

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the server
    Serve,
    /// Run migrations against the database
    Migrations {
        #[command(subcommand)]
        command: MigrationsCommand,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv();

    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_ansi(true)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");

    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Serve);

    match command {
        Commands::Serve => application::start_server().await?,
        Commands::Migrations { command } => {
            crate::application::migrations::run_migrations(command).await?;
        }
    }

    Ok(())
}
