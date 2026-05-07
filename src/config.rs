use std::fs;

use serde::{Deserialize, Serialize};

use crate::db::default_config_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_language: String,
    pub theme: String,
    pub max_history: usize,
    pub show_related: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_language: "rust".to_string(),
            theme: "dark".to_string(),
            max_history: 100,
            show_related: true,
        }
    }
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    let config_dir = default_config_dir()?;
    fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("config.toml");
    if !config_path.exists() {
        let config = Config::default();
        fs::write(&config_path, toml::to_string_pretty(&config)?)?;
        return Ok(config);
    }

    let text = fs::read_to_string(config_path)?;
    Ok(toml::from_str(&text).unwrap_or_default())
}
