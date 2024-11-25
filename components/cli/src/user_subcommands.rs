use clap::Subcommand;

use crate::config::Config;

#[derive(Subcommand)]
pub enum UserCommands {
    /// Print information about the current user
    Get,

}

pub async fn handle_user_command(command: UserCommands) -> anyhow::Result<()> {
    match command {
        UserCommands::Get => {
            let config = Config::load().await?;
            let client = config.get_api_client()?;
            let user = client.auth().get_current_user().await?;
            println!("{:?}", user);
            Ok(())
        }
    }
}
