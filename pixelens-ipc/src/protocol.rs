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
/// duplicate the schema. M6 validates per-command payloads at the
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

impl IpcResponse {
    /// Build a successful `ok` response with the given payload.
    pub fn ok(request_id: RequestId, payload: impl Serialize) -> Result<Self, serde_json::Error> {
        Ok(Self {
            request_id,
            status: ResponseStatus::Ok,
            payload: serde_json::to_value(payload)?,
        })
    }

    /// Build an `error` response with the given error message.
    pub fn error(request_id: RequestId, message: impl Into<String>) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Error,
            payload: serde_json::json!({ "error": message.into() }),
        }
    }

    /// Build a `cancelled` response.
    pub fn cancelled(request_id: RequestId) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Cancelled,
            payload: serde_json::json!({ "reason": "cancelled" }),
        }
    }
}

/// Payload returned by the `grab` command on success.
///
/// Region coordinates use the same `(x, y, width, height)` convention as
/// the slurp/grim geometry string (`WxH+X+Y`). `path` is an absolute
/// filesystem path; the CLI is responsible for any further handling
/// (display, OCR, deletion). The file persists until the caller removes
/// it or the temp directory is cleaned.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrabResponsePayload {
    pub path: String,
    pub region: GrabRegion,
    pub bytes: u64,
    /// Unix epoch milliseconds at which the capture completed.
    pub captured_at_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrabRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl From<pixelens_core::Rect> for GrabRegion {
    fn from(r: pixelens_core::Rect) -> Self {
        Self {
            x: r.origin.x,
            y: r.origin.y,
            width: r.size.width,
            height: r.size.height,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_response_carries_serialized_payload() {
        let payload = GrabResponsePayload {
            path: "/tmp/x.png".to_string(),
            region: GrabRegion {
                x: 10,
                y: 20,
                width: 100,
                height: 50,
            },
            bytes: 12345,
            captured_at_ms: 1700000000000,
        };
        let resp = IpcResponse::ok("req-1".to_string(), &payload).unwrap();
        assert_eq!(resp.status, ResponseStatus::Ok);
        assert_eq!(resp.payload["path"], "/tmp/x.png");
        assert_eq!(resp.payload["bytes"], 12345);
    }

    #[test]
    fn grab_payload_round_trips() {
        let payload = GrabResponsePayload {
            path: "/tmp/a.png".to_string(),
            region: GrabRegion {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            bytes: 0,
            captured_at_ms: 0,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let back: GrabResponsePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(back, payload);
    }

    #[test]
    fn error_response_includes_message() {
        let resp = IpcResponse::error("req-1".to_string(), "boom");
        assert_eq!(resp.status, ResponseStatus::Error);
        assert_eq!(resp.payload["error"], "boom");
    }

    #[test]
    fn cancelled_response_marks_status() {
        let resp = IpcResponse::cancelled("req-1".to_string());
        assert_eq!(resp.status, ResponseStatus::Cancelled);
        assert_eq!(resp.payload["reason"], "cancelled");
    }
}
