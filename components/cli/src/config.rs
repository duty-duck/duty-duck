use std::path::PathBuf;

use anyhow::Context;
use api_client_rs::DutyDuckApiClient;
use dirs::config_dir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_url: String,
    pub api_token_id: Option<String>,
    pub api_token_secret_key: Option<String>,
}

impl Config {
    pub fn get_api_client(&self) -> anyhow::Result<DutyDuckApiClient> {
        let client = DutyDuckApiClient::new();
        if let Some(api_token_id) = self.api_token_id.as_ref() {
            client.set_api_token_id(api_token_id.clone())?;
        }
        if let Some(api_token_secret_key) = self.api_token_secret_key.as_ref() {
            client.set_api_token_secret_key(api_token_secret_key.clone())?;
        }
        Ok(client)
    }

    pub async fn load() -> anyhow::Result<Config> {
        match get_config_from_file().await {
            Ok(config) => Ok(config),
            Err(e) => {
                eprintln!("Failed to get config from file. Creating a new config file.");
                let config = Config::default();
                config.save().await?;
                Ok(config)
            }
        }
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let config_file = get_config_file()?;
        let config_dir = config_file
            .parent()
            .context("Failed to get config directory")?;
        tokio::fs::create_dir_all(config_dir)
            .await
            .context("Failed to create config directory")?;
        let serialized = serde_json::to_string_pretty(self).context("Failed to serialize config")?;
        tokio::fs::write(config_file, serialized)
            .await
            .context("Failed to save config file")
    }

    pub fn get(&self, key: &str) -> anyhow::Result<&str> {
        match key {
            "api_url" => Ok(&self.api_url),
            "api_token_id" => Ok(self.api_token_id.as_deref().unwrap_or("null")),
            "api_token_secret_key" => Ok(self.api_token_secret_key.as_deref().unwrap_or("null")),
            _ => Err(anyhow::anyhow!("Invalid key: {}", key)),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        match key {
            "api_url" => self.api_url = value.to_string(),
            "api_token_id" => self.api_token_id = Some(value.to_string()),
            "api_token_secret_key" => self.api_token_secret_key = Some(value.to_string()),
            _ => return Err(anyhow::anyhow!("Invalid key: {}", key)),
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: "https://api.dutyduck.net".to_string(),
            api_token_id: None,
            api_token_secret_key: None,
        }
    }
}

fn get_config_dir() -> anyhow::Result<PathBuf> {
    let dir = config_dir()
        .context("Failed to get config directory")?
        .join("dutyduck");
    Ok(dir)
}

fn get_config_file() -> anyhow::Result<PathBuf> {
    let dir = get_config_dir()?;
    Ok(dir.join("config.json"))
}

async fn get_config_from_file() -> anyhow::Result<Config> {
    let file = get_config_file()?;
    let config = std::fs::read_to_string(file).context("Failed to read config file")?;
    let config: Config = serde_json::from_str(&config).context("Failed to parse config file")?;
    Ok(config)
}
