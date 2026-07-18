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
    fn get_and_set_dotted_keys() {
        let mut cfg = Config::default();
        assert_eq!(get_value(&cfg, "general.hotkey").unwrap(), "Super+Shift+T");
        set_value(&mut cfg, "general.hotkey", "Super+Shift+S").unwrap();
        assert_eq!(get_value(&cfg, "general.hotkey").unwrap(), "Super+Shift+S");
        // unknown key errors
        assert!(get_value(&cfg, "nope.nope").is_err());
        assert!(set_value(&mut cfg, "nope.nope", "x").is_err());
        // bad bool errors
        assert!(set_value(&mut cfg, "capture.show_preview", "maybe").is_err());
    }
}
