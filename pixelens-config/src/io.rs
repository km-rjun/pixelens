//! File I/O for Pixelens configuration.
//!
//! Resolves the on-disk `config.toml`, loads it into the typed
//! [`Config`] (returning defaults when the file is absent or
//! unreadable), and writes it back out. This is the glue that makes
//! the configuration model *used* rather than merely defined (M8).

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::model::Config;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse error: {0}")]
    Toml(String),

    #[error("unknown config key: {0}")]
    UnknownKey(String),

    #[error("invalid value for {key}: {reason}")]
    InvalidValue { key: String, reason: String },
}

/// The dotted keys we recognise. Mirrors [`Config`]'s shape.
pub const KNOWN_KEYS: &[&str] = &[
    "general.autostart",
    "general.theme",
    "general.hotkey",
    "capture.show_preview",
    "ocr.engine",
    "ocr.language",
    "ai.endpoint",
    "ai.api_key",
    "ai.model",
    "ai.menu_backend",
    "ai.require_key",
    "search.provider",
    "upload.endpoint",
    "upload.provider",
    "reverse_image.provider",
];

/// Resolve the default config path: `~/.config/pixelens/config.toml`.
///
/// Honours `XDG_CONFIG_HOME` when set and non-empty; otherwise falls
/// back to `$HOME/.config`. Returns a synthetic path when neither is
/// available (tests override via [`load_config_from`]).
pub fn config_path() -> PathBuf {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("pixelens").join("config.toml");
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("pixelens")
            .join("config.toml");
    }
    PathBuf::from(".config")
        .join("pixelens")
        .join("config.toml")
}

/// Load [`Config`] from the default path. Missing/invalid files do
/// **not** error — they fall back to defaults so the daemon and CLI
/// always have a usable config object.
pub fn load_config() -> Result<Config, ConfigError> {
    load_config_from(&config_path())
}

/// Load [`Config`] from an explicit path. Absent files return defaults.
pub fn load_config_from(path: &Path) -> Result<Config, ConfigError> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let raw = std::fs::read_to_string(path)?;
    let cfg: Config = toml::from_str(&raw).map_err(|e| ConfigError::Toml(e.to_string()))?;
    Ok(cfg)
}

/// Serialise and write [`Config`] to the default path, creating the
/// parent directory if needed.
pub fn save_config(cfg: &Config) -> Result<(), ConfigError> {
    save_config_to(&config_path(), cfg)
}

/// Serialise and write [`Config`] to an explicit path (used by tests).
pub fn save_config_to(path: &Path, cfg: &Config) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let serialized = toml::to_string_pretty(cfg).map_err(|e| ConfigError::Toml(e.to_string()))?;
    std::fs::write(path, serialized)?;
    Ok(())
}

/// Get the string representation of a dotted key.
pub fn get_value(cfg: &Config, key: &str) -> Result<String, ConfigError> {
    match key {
        "general.autostart" => Ok(cfg.general.autostart.to_string()),
        "general.theme" => Ok(cfg.general.theme.clone()),
        "general.hotkey" => Ok(cfg.general.hotkey.clone()),
        "capture.show_preview" => Ok(cfg.capture.show_preview.to_string()),
        "ocr.engine" => Ok(cfg.ocr.engine.clone()),
        "ocr.language" => Ok(cfg.ocr.language.clone()),
        "ai.endpoint" => Ok(cfg.ai.endpoint.clone()),
        "ai.api_key" => Ok(cfg.ai.api_key.clone()),
        "ai.model" => Ok(cfg.ai.model.clone()),
        "ai.menu_backend" => Ok(cfg.ai.menu_backend.clone()),
        "ai.require_key" => Ok(cfg.ai.require_key.to_string()),
        "search.provider" => Ok(cfg.search.provider.clone()),
        "upload.endpoint" => Ok(cfg.upload.endpoint.clone()),
        "upload.provider" => Ok(cfg.upload.provider.clone()),
        "reverse_image.provider" => Ok(cfg.reverse_image.provider.clone()),
        other => Err(ConfigError::UnknownKey(other.to_string())),
    }
}

/// Set a dotted key from a raw string, validating the type loosely.
pub fn set_value(cfg: &mut Config, key: &str, value: &str) -> Result<(), ConfigError> {
    match key {
        "general.autostart" => {
            cfg.general.autostart = parse_bool(key, value)?;
        }
        "general.theme" => {
            cfg.general.theme = value.to_string();
        }
        "general.hotkey" => {
            cfg.general.hotkey = value.to_string();
        }
        "capture.show_preview" => {
            cfg.capture.show_preview = parse_bool(key, value)?;
        }
        "ocr.engine" => {
            cfg.ocr.engine = value.to_string();
        }
        "ocr.language" => {
            cfg.ocr.language = value.to_string();
        }
        "ai.endpoint" => {
            cfg.ai.endpoint = value.to_string();
        }
        "ai.api_key" => {
            cfg.ai.api_key = value.to_string();
        }
        "ai.model" => {
            cfg.ai.model = value.to_string();
        }
        "ai.menu_backend" => {
            cfg.ai.menu_backend = value.to_string();
        }
        "ai.require_key" => {
            cfg.ai.require_key = parse_bool(key, value)?;
        }
        "search.provider" => {
            cfg.search.provider = value.to_string();
        }
        "upload.endpoint" => {
            cfg.upload.endpoint = value.to_string();
        }
        "upload.provider" => {
            cfg.upload.provider = value.to_string();
        }
        "reverse_image.provider" => {
            cfg.reverse_image.provider = value.to_string();
        }
        other => return Err(ConfigError::UnknownKey(other.to_string())),
    }
    Ok(())
}

fn parse_bool(key: &str, value: &str) -> Result<bool, ConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        other => Err(ConfigError::InvalidValue {
            key: key.to_string(),
            reason: format!("'{other}' is not a boolean (expected true/false)"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("pixelens-test-{name}-{}.toml", std::process::id()))
    }

    #[test]
    fn load_defaults_when_missing() {
        let path = temp_path("missing");
        let _ = std::fs::remove_file(&path);
        let cfg = load_config_from(&path).expect("load should succeed for missing file");
        assert!(!cfg.general.autostart);
        assert_eq!(cfg.general.theme, "system");
        assert_eq!(cfg.general.hotkey, "Super+Shift+T");
        assert!(!cfg.capture.show_preview);
        assert_eq!(cfg.ocr.engine, "tesseract");
        assert_eq!(cfg.ocr.language, "eng");
        assert_eq!(cfg.ai.endpoint, "http://10.0.0.1:11434/v1");
        assert_eq!(cfg.ai.api_key, "");
        assert_eq!(cfg.ai.model, "llava");
        assert_eq!(cfg.ai.menu_backend, "fuzzel");
        assert!(!cfg.ai.require_key);
        assert_eq!(cfg.search.provider, "google_lens");
        assert_eq!(cfg.upload.endpoint, "");
        assert_eq!(cfg.upload.provider, "zeroxzero");
        assert_eq!(cfg.reverse_image.provider, "google_lens");
    }

    #[test]
    fn load_round_trips() {
        let path = temp_path("roundtrip");
        let mut cfg = Config::default();
        set_value(&mut cfg, "general.hotkey", "Ctrl+Alt+G").unwrap();
        set_value(&mut cfg, "capture.show_preview", "true").unwrap();
        set_value(&mut cfg, "general.theme", "dark").unwrap();
        save_config_to(&path, &cfg).unwrap();

        let loaded = load_config_from(&path).unwrap();
        assert_eq!(loaded.general.hotkey, "Ctrl+Alt+G");
        assert!(loaded.capture.show_preview);
        assert_eq!(loaded.general.theme, "dark");
        // Unchanged defaults preserved.
        assert!(!loaded.general.autostart);
        assert_eq!(loaded.ocr.engine, "tesseract");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn ai_search_upload_keys_round_trip() {
        let path = temp_path("ai");
        let mut cfg = Config::default();
        set_value(&mut cfg, "ai.endpoint", "https://api.example/v1").unwrap();
        set_value(&mut cfg, "ai.model", "gpt-4o").unwrap();
        set_value(&mut cfg, "ai.menu_backend", "wofi").unwrap();
        set_value(&mut cfg, "ai.require_key", "true").unwrap();
        set_value(&mut cfg, "ocr.language", "spa").unwrap();
        set_value(&mut cfg, "search.provider", "google_lens").unwrap();
        set_value(&mut cfg, "reverse_image.provider", "google_lens").unwrap();
        save_config_to(&path, &cfg).unwrap();

        let loaded = load_config_from(&path).unwrap();
        assert_eq!(loaded.ai.endpoint, "https://api.example/v1");
        assert_eq!(loaded.ai.model, "gpt-4o");
        assert_eq!(loaded.ai.menu_backend, "wofi");
        assert!(loaded.ai.require_key);
        assert_eq!(loaded.ocr.language, "spa");
        assert_eq!(loaded.search.provider, "google_lens");
        assert_eq!(loaded.reverse_image.provider, "google_lens");

        // get_value reflects the same
        assert_eq!(
            get_value(&loaded, "ai.endpoint").unwrap(),
            "https://api.example/v1"
        );
        // bad bool still errors
        assert!(set_value(&mut cfg, "ai.require_key", "maybe").is_err());
        // unknown key still errors
        assert!(set_value(&mut cfg, "ai.nope", "x").is_err());

        let _ = std::fs::remove_file(&path);
    }
}
