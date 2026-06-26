use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_endpoint: String,
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
    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("pixelens").join("config.json")
    }
    
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let data = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }
    
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {}", e))?;
        let data = serde_json::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&path, data).map_err(|e| format!("Failed to write config: {}", e))
    }
}
