use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub model: String,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        let dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".bitart");
        fs::create_dir_all(&dir).ok();
        dir
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Option<Config> {
        let path = Self::config_path();
        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(path, data).map_err(|e| format!("Failed to write config: {}", e))
    }

    pub fn available_models() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("dall-e-2", "DALL-E 2", "$0.02/image (cheapest)"),
            ("dall-e-3", "DALL-E 3", "$0.04/image"),
            ("gpt-image-1", "GPT Image 1", "$0.04/image (best quality)"),
        ]
    }
}
