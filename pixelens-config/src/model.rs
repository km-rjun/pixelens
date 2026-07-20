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
    /// OCR language code (e.g. "eng", "spa"). Migrated from `main`'s
    /// `ocr_language` during unification (2026-07-20, Strategy C).
    #[serde(default = "default_ocr_language")]
    pub language: String,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            engine: default_ocr_engine(),
            language: default_ocr_language(),
        }
    }
}

/// AI / LLM settings. Ported from `main`'s config (Strategy C, 2026-07-20).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// OpenAI-compatible base URL. Defaults to the local Ollama endpoint.
    #[serde(default = "default_ai_endpoint")]
    pub endpoint: String,
    /// API key. Empty for Ollama; required for hosted OpenAI-compatible servers.
    #[serde(default = "default_ai_api_key")]
    pub api_key: String,
    /// Default model. `llava` is vision-capable and ships with Ollama.
    #[serde(default = "default_ai_model")]
    pub model: String,
    /// Action-bar backend: `fuzzel` | `wofi` | `stdin`.
    #[serde(default = "default_ai_menu_backend")]
    pub menu_backend: String,
    /// When false, the AI client skips API-key validation (Ollama case).
    #[serde(default = "default_ai_require_key")]
    pub require_key: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            endpoint: default_ai_endpoint(),
            api_key: default_ai_api_key(),
            model: default_ai_model(),
            menu_backend: default_ai_menu_backend(),
            require_key: default_ai_require_key(),
        }
    }
}

/// Web-search provider settings (reverse-image + text search).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Search backend. Currently only `google_lens` is supported.
    #[serde(default = "default_search_provider")]
    pub provider: String,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            provider: default_search_provider(),
        }
    }
}

/// Custom image-upload backend (used by upload + reverse-image flows).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    /// Upload endpoint URL (empty = use the default hosted uploader).
    #[serde(default = "default_upload_endpoint")]
    pub endpoint: String,
    /// Upload provider name.
    #[serde(default = "default_upload_provider")]
    pub provider: String,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            endpoint: default_upload_endpoint(),
            provider: default_upload_provider(),
        }
    }
}

/// Reverse-image-search provider settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseImageConfig {
    /// Backend used for "search by image". Defaults to the search provider.
    #[serde(default = "default_reverse_image_provider")]
    pub provider: String,
}

impl Default for ReverseImageConfig {
    fn default() -> Self {
        Self {
            provider: default_reverse_image_provider(),
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
    /// Unified AI/LLM settings (Strategy C, 2026-07-20).
    #[serde(default)]
    pub ai: AiConfig,
    /// Unified web-search settings (Strategy C).
    #[serde(default)]
    pub search: SearchConfig,
    /// Unified image-upload settings (Strategy C).
    #[serde(default)]
    pub upload: UploadConfig,
    /// Unified reverse-image-search settings (Strategy C).
    #[serde(default)]
    pub reverse_image: ReverseImageConfig,
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
fn default_ocr_language() -> String {
    "eng".to_string()
}
fn default_ai_endpoint() -> String {
    "http://10.0.0.1:11434/v1".to_string()
}
fn default_ai_api_key() -> String {
    String::new()
}
fn default_ai_model() -> String {
    "llava".to_string()
}
fn default_ai_menu_backend() -> String {
    "fuzzel".to_string()
}
fn default_ai_require_key() -> bool {
    false
}
fn default_search_provider() -> String {
    "google_lens".to_string()
}
fn default_upload_endpoint() -> String {
    String::new()
}
fn default_upload_provider() -> String {
    "zeroxzero".to_string()
}
fn default_reverse_image_provider() -> String {
    "google_lens".to_string()
}
fn default_hud_enabled() -> bool {
    true
}
fn default_hud_timeout_ms() -> u64 {
    1500
}
