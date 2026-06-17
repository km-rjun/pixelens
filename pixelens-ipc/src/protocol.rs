//! Wire protocol messages exchanged between CLI and daemon.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Opaque session/request identifier. Always a UUIDv4 string on the wire.
pub type RequestId = String;

pub const CANCEL_COMMAND: &str = "cancel";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Command {
    Grab,
    Status,
    Stop,
    ConfigGet,
    ConfigSet,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Ok,
    Error,
    Cancelled,
}

/// Request frame sent from CLI to daemon.
///
/// `payload` is intentionally `serde_json::Value` for v1: each command
/// defines its own payload shape, and centralising the union here would
/// duplicate the schema. M6 will validate per-command payloads at the
/// dispatch layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub request_id: RequestId,
    pub command: Command,
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// Response frame sent from daemon to CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub request_id: RequestId,
    pub status: ResponseStatus,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("invalid frame: {0}")]
    InvalidFrame(String),

    #[error("frame too large ({size} bytes, max {max})")]
    FrameTooLarge { size: usize, max: usize },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
