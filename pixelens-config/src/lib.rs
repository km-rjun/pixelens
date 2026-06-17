//! `config.toml` types and defaults.
//!
//! Mirrors the schema in PRD §"Configuration". The actual file I/O,
//! path resolution, and `pixelens config` CLI subcommands land in M8.
//! M1 just defines the data model so other crates can refer to
//! `Config` without circular deps.

pub mod model;

pub use model::{CaptureConfig, Config, GeneralConfig, OcrConfig};
