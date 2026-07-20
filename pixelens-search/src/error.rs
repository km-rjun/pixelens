//! Error type for the `pixelens-search` crate.

use thiserror::Error;

/// Errors produced by search / reverse-image / upload operations.
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("upload failed: {0}")]
    Upload(String),

    #[error("network request failed: {0}")]
    Network(String),

    #[error("invalid response: {0}")]
    InvalidResponse(String),
}
