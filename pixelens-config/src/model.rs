//! Strongly-typed configuration model.
//!
//! Defaults match PRD §"Configuration" verbatim. `show_preview` defaults
//! to `false` so the fast path is guaranteed for any user who has not
//! explicitly opted in.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_autostart")]
    pub autostart: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            autostart: default_autostart(),
            theme: default_theme(),
            hotkey: default_hotkey(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    #[serde(default = "default_show_preview")]
    pub show_preview: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            show_preview: default_show_preview(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrConfig {
    #[serde(default = "default_ocr_engine")]
    pub engine: String,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            engine: default_ocr_engine(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub capture: CaptureConfig,
    #[serde(default)]
    pub ocr: OcrConfig,
}

fn default_autostart() -> bool {
    false
}
fn default_theme() -> String {
    "system".to_string()
}
fn default_hotkey() -> String {
    "Super+Shift+T".to_string()
}
fn default_show_preview() -> bool {
    false
}
fn default_ocr_engine() -> String {
    "tesseract".to_string()
}
