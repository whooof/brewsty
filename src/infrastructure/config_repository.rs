use crate::domain::entities::AppConfig;
use anyhow::{Context, Result};
use dirs::home_dir;
use std::fs;
use std::path::PathBuf;

pub struct ConfigRepository {
    config_path: PathBuf,
}

impl ConfigRepository {
    pub fn new() -> Self {
        let mut config_dir = home_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.push(".brewsty");

        Self {
            config_path: config_dir.join("config.toml"),
        }
    }

    pub fn load(&self) -> Result<AppConfig> {
        if !self.config_path.exists() {
            return Ok(AppConfig::default());
        }

        let content =
            fs::read_to_string(&self.config_path).context("Failed to read config file")?;

        let config = toml::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self, config: &AppConfig) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(config).context("Failed to serialize config")?;

        fs::write(&self.config_path, content).context("Failed to write config file")?;

        Ok(())
    }

    /// Get the config file path
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}
