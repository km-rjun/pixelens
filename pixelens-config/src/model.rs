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
    /// UM4: HUD / GUI feature flags. Present even though the visual HUD
    /// crate (`pixelens-gui`) is deferred; `hud_enabled` acts as the
    /// master switch the future HUD will read, and `hud_timeout_ms`
    /// tunes auto-dismiss.
    #[serde(default)]
    pub gui: GuiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    /// Master switch for the UM4 HUD feature. When `false`, the hotkey
    /// `Space` chord is ignored and behaviour is identical to v1.0.
    #[serde(default = "default_hud_enabled")]
    pub hud_enabled: bool,
    /// Auto-dismiss timeout for the HUD in milliseconds.
    #[serde(default = "default_hud_timeout_ms")]
    pub hud_timeout_ms: u64,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            hud_enabled: default_hud_enabled(),
            hud_timeout_ms: default_hud_timeout_ms(),
        }
    }
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
fn default_hud_enabled() -> bool {
    true
}
fn default_hud_timeout_ms() -> u64 {
    1500
}
