use std::fs;
use std::path::PathBuf;

use crate::error::PixelensError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_endpoint: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub api_key: String,
    pub model: String,
    pub ocr_language: String,
    pub hotkey: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_endpoint: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o".to_string(),
            ocr_language: "eng".to_string(),
            hotkey: "Ctrl+Shift+C".to_string(),
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("pixelens").join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        let mut config = if path.exists() {
            let data = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        };

        // Check environment variable for API key
        if let Ok(env_key) = std::env::var("PIXELENS_API_KEY") {
            config.api_key = env_key;
        }

        config
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn save(&self) -> Result<(), PixelensError> {
        let path = Self::config_path();
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent)?;

        // Don't save API key to file if it came from environment
        let mut save_config = self.clone();
        if std::env::var("PIXELENS_API_KEY").is_ok() {
            save_config.api_key = String::new();
        }

        let data = serde_json::to_string_pretty(&save_config)?;
        fs::write(&path, data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.api_endpoint, "https://api.openai.com/v1");
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.ocr_language, "eng");
    }

    #[test]
    fn test_config_path() {
        let path = Config::config_path();
        assert!(path.to_string_lossy().contains("pixelens"));
        assert!(path.to_string_lossy().contains("config.json"));
    }
}
