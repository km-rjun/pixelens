//! Crate-wide error type.
//!
//! One enum, used by every Pixelens crate. `thiserror` generates the
//! `Display` and `Error` impls; concrete variants are added as later
//! milestones land (capture backends, OCR errors, IPC failures, etc.).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PixelensError {
    #[error("no display server detected (neither WAYLAND_DISPLAY nor DISPLAY is set)")]
    NoDisplayServer,

    #[error("unsupported display server")]
    UnsupportedDisplayServer,

    #[error("capture failed: {0}")]
    Capture(String),

    #[error("OCR failed: {0}")]
    Ocr(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}

pub type PixelensResult<T> = Result<T, PixelensError>;
