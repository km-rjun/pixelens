use std::fs;
use std::path::PathBuf;

use crate::error::PixelensError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub api_endpoint: String,
    pub api_key: String,
    pub model: String,
    pub ocr_language: String,
    pub hotkey: String,
    pub menu_backend: String,
    pub image_upload_provider: String,
    pub reverse_image_provider: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_endpoint: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o".to_string(),
            ocr_language: "eng".to_string(),
            hotkey: "Ctrl+Shift+C".to_string(),
            menu_backend: "auto".to_string(),
            image_upload_provider: String::new(),
            reverse_image_provider: "google_lens".to_string(),
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
            match serde_json::from_str::<Config>(&data) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Failed to parse config at {}: {}", path.display(), e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        };

        if let Ok(val) = std::env::var("PIXELENS_API_KEY") {
            config.api_key = val;
        }
        if let Ok(val) = std::env::var("PIXELENS_MODEL") {
            config.model = val;
        }
        if let Ok(val) = std::env::var("PIXELENS_API_ENDPOINT") {
            config.api_endpoint = val;
        }
        if let Ok(val) = std::env::var("PIXELENS_OCR_LANGUAGE") {
            config.ocr_language = val;
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
        assert_eq!(config.menu_backend, "auto");
        assert_eq!(config.image_upload_provider, "");
        assert_eq!(config.reverse_image_provider, "google_lens");
    }

    #[test]
    fn test_config_path() {
        let path = Config::config_path();
        assert!(path.to_string_lossy().contains("pixelens"));
        assert!(path.to_string_lossy().contains("config.json"));
    }

    #[test]
    fn test_model_from_file() {
        let json = r#"{"model": "gpt-5.4-mini"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.model, "gpt-5.4-mini");
    }

    #[test]
    fn test_partial_file_uses_defaults() {
        let json = r#"{"model": "custom-model"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.model, "custom-model");
        assert_eq!(config.ocr_language, "eng");
    }

    #[test]
    fn test_env_override_model() {
        std::env::set_var("PIXELENS_MODEL", "env-model");
        let mut config = Config::default();
        if let Ok(val) = std::env::var("PIXELENS_MODEL") {
            config.model = val;
        }
        assert_eq!(config.model, "env-model");
        std::env::remove_var("PIXELENS_MODEL");
    }

    #[test]
    fn test_env_does_not_override_unset() {
        std::env::remove_var("PIXELENS_MODEL");
        let mut config = Config {
            model: "file-model".to_string(),
            ..Default::default()
        };
        if let Ok(val) = std::env::var("PIXELENS_MODEL") {
            config.model = val;
        }
        assert_eq!(config.model, "file-model");
    }

    #[test]
    fn test_defaults_only_when_no_file() {
        let config = Config::default();
        assert_eq!(config.model, "gpt-4o");
    }
}
