#[macro_use]
extern crate rust_i18n;

use application::{background_tasks::BackgroundTask, migrations::MigrationsCommand};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use clap::*;

mod application;
mod domain;
mod infrastructure;
mod shared;
mod protos;

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
    /// Run a background task manually
    Run {
       #[command(subcommand)]
       task: BackgroundTask,
    },
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
        .with_env_filter(EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");


    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Serve);

    match command {
        Commands::Serve => application::start_server().await?,
        Commands::Run { task } => {
            tracing::info!("Running background task: {:?}", task);
            crate::application::background_tasks::run_background_task(task).await?;
        }
        Commands::Migrations { command } => {
            crate::application::migrations::run_migrations(command).await?;
        }
    }
    
    Ok(())
}
