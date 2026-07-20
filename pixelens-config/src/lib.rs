//! `config.toml` types and defaults.
//!
//! Mirrors the schema in PRD §"Configuration". The actual file I/O,
//! path resolution, and `pixelens config` CLI subcommands land in M8.
//! M1 just defines the data model so other crates can refer to
//! `Config` without circular deps.

pub mod io;
pub mod model;

pub use io::{
    config_path, get_value, load_config, load_config_from, save_config, save_config_to, set_value,
    ConfigError, KNOWN_KEYS,
};
pub use model::{
    AiConfig, CaptureConfig, Config, GeneralConfig, GuiConfig, OcrConfig, ReverseImageConfig,
    SearchConfig, UploadConfig,
};
