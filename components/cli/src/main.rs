use clap::{Parser, Subcommand};

mod config;
mod config_subcommands;
mod user_subcommands;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { command } => {
            config_subcommands::handle_config_command(command).await
        }
        Commands::User { command } => {
            user_subcommands::handle_user_command(command).await
        }
    }
}
