use clap::Subcommand;

use crate::config::Config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Print the current configuration
    Print,
    /// Get a configuration value
    Get {
        /// Name of the configuration key
        key: String,
    },
    /// Set a configuration value
    Set {
        /// Name of the configuration key
        key: String,
        /// Value to set
        value: String,
    },
}

pub async fn handle_config_command(command: ConfigCommands) -> anyhow::Result<()> {
    match command {
        ConfigCommands::Get { key } => {
            let config = Config::load().await?;
            println!("{}", config.get(&key)?);
            Ok(())
        }
        ConfigCommands::Set { key, value } => {
            let mut config = Config::load().await?;
            config.set(&key, &value)?;
            config.save().await?;
            println!("Config updated. Set {} = {}", key, value);
            Ok(())
        }
        ConfigCommands::Print => {
            let config = Config::load().await?;
            println!("Current configuration: {}", serde_json::to_string_pretty(&config)?);
            Ok(())
        }
    }
}
