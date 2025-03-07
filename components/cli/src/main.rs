use clap::{Parser, Subcommand};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod config;
mod config_subcommands;
mod tasks_subcommands;
mod user_subcommands;

#[derive(Parser)]
#[command(author, version, about, long_about = None, name = "dutyduck")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configuration related commands
    Config {
        #[command(subcommand)]
        command: config_subcommands::ConfigCommands,
    },
    /// User related commands
    User {
        #[command(subcommand)]
        command: user_subcommands::UserCommands,
    },
    /// Tasks related commands
    Tasks {
        #[command(subcommand)]
        command: tasks_subcommands::TasksCommands,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    match cli.command {
        Commands::Config { command } => config_subcommands::handle_config_command(command).await,
        Commands::User { command } => user_subcommands::handle_user_command(command).await,
        Commands::Tasks { command } => tasks_subcommands::handle_tasks_command(command).await,
    }
}
