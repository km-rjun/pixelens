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
    /// UM4: re-run display/output detection before the next grab.
    Redetect,
    /// UM4: one-shot preview override for the *next* grab only. `payload`
    /// carries the boolean; subsequent grabs fall back to config.
    SetPreview,
    /// u5: ask the configured OpenAI-compatible model (e.g. Ollama) a
    /// question. `payload` carries [`AiPayload`]; the response embeds the
    /// model's text in [`AiResponsePayload`].
    Ai,
    /// u6: build a web-search URL from OCR/selected text. `payload`
    /// carries [`SearchPayload`]; the response embeds the URL in
    /// [`SearchResponsePayload`]. No network call is made by the daemon —
    /// the client opens the URL.
    Search,
    /// u6: run a "search by image" flow: optionally upload the capture to
    /// the configured image host, then build a Google Lens URL. `payload`
    /// carries [`ReverseImagePayload`]; the response reuses
    /// [`AiResponsePayload`] for the status string.
    ReverseImage,
    /// u6: translate text via the configured model. `payload` carries
    /// [`TranslatePayload`]; the response reuses [`AiResponsePayload`].
    Translate,
}

/// Payload for [`Command::SetPreview`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetPreviewPayload {
    /// When `true`, force a capture preview for the next grab; when
    /// `false`, suppress it. After one grab the override is cleared and
    /// the daemon reverts to `capture.show_preview` from config.
    pub preview: bool,
}

/// Payload for [`Command::Ai`].
///
/// `prompt` is the user's question (often the OCR text of a capture plus
/// an instruction). `image_path`, when present, is an absolute path to a
/// PNG the model may inspect if it supports vision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiPayload {
    pub prompt: String,
    /// Optional absolute path to an image for vision-capable models.
    #[serde(default)]
    pub image_path: Option<String>,
}

/// Response payload for [`Command::Ai`] on success.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiResponsePayload {
    /// The model's reply text.
    pub text: String,
    /// The model id that produced the reply (echoed from config).
    pub model: String,
}

/// Payload for [`Command::Search`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchPayload {
    /// Free text (usually OCR output of a capture) to search for.
    pub text: String,
}

/// Response payload for [`Command::Search`] on success.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResponsePayload {
    /// The fully-qualified search URL the client should open.
    pub url: String,
}

/// Payload for [`Command::ReverseImage`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReverseImagePayload {
    /// Absolute path to the captured image to search by image.
    pub image_path: String,
}

/// Payload for [`Command::Translate`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslatePayload {
    /// Text to translate (usually OCR output of a capture).
    pub text: String,
    /// Target language name or code, e.g. "Spanish" or "fr".
    pub target_lang: String,
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
    /// Extracted text from the captured region (M5+). Empty string when
    /// OCR is unavailable or found no text. Defaults to `""` on the wire
    /// so older readers still deserialise captures produced before M5.
    #[serde(default)]
    pub text: String,
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
            text: String::new(),
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
            text: String::new(),
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

    #[test]
    fn redetect_command_serializes_lowercase() {
        let json = serde_json::to_string(&Command::Redetect).unwrap();
        assert_eq!(json, "\"redetect\"");
        let back: Command = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Command::Redetect);
    }

    #[test]
    fn ai_command_and_payload_round_trip() {
        let json = serde_json::to_string(&Command::Ai).unwrap();
        assert_eq!(json, "\"ai\"");
        let back: Command = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Command::Ai);

        let payload = AiPayload {
            prompt: "what is this?".to_string(),
            image_path: Some("/tmp/x.png".to_string()),
        };
        let pj = serde_json::to_string(&payload).unwrap();
        let pback: AiPayload = serde_json::from_str(&pj).unwrap();
        assert_eq!(pback, payload);

        // image_path defaults to None when omitted on the wire.
        let sparse: AiPayload = serde_json::from_str(r#"{"prompt":"hi"}"#).unwrap();
        assert_eq!(sparse.prompt, "hi");
        assert!(sparse.image_path.is_none());
    }

    #[test]
    fn ai_response_payload_serializes() {
        let payload = AiResponsePayload {
            text: "42".to_string(),
            model: "llava".to_string(),
        };
        let resp = IpcResponse::ok("req-9".to_string(), &payload).unwrap();
        assert_eq!(resp.status, ResponseStatus::Ok);
        assert_eq!(resp.payload["text"], "42");
        assert_eq!(resp.payload["model"], "llava");
    }

    #[test]
    fn u6_search_and_translate_round_trip() {
        for (cmd, json) in [
            (Command::Search, "\"search\""),
            (Command::ReverseImage, "\"reverseimage\""),
            (Command::Translate, "\"translate\""),
        ] {
            let s = serde_json::to_string(&cmd).unwrap();
            assert_eq!(s, json);
            let back: Command = serde_json::from_str(json).unwrap();
            assert_eq!(back, cmd);
        }

        let sp = SearchPayload {
            text: "hello world".to_string(),
        };
        let spj = serde_json::to_string(&sp).unwrap();
        assert_eq!(serde_json::from_str::<SearchPayload>(&spj).unwrap(), sp);

        let rp = ReverseImagePayload {
            image_path: "/tmp/x.png".to_string(),
        };
        assert_eq!(
            serde_json::from_str::<ReverseImagePayload>(&serde_json::to_string(&rp).unwrap())
                .unwrap(),
            rp
        );

        let tp = TranslatePayload {
            text: "bonjour".to_string(),
            target_lang: "en".to_string(),
        };
        assert_eq!(
            serde_json::from_str::<TranslatePayload>(&serde_json::to_string(&tp).unwrap()).unwrap(),
            tp
        );

        let sresp = SearchResponsePayload {
            url: "https://www.google.com/search?q=hi".to_string(),
        };
        let r = IpcResponse::ok("req-s".to_string(), &sresp).unwrap();
        assert_eq!(r.payload["url"], "https://www.google.com/search?q=hi");
    }
}
