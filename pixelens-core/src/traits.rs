//! Core traits declared in the PRD.
//!
//! `CaptureProvider` and `OcrEngine` are introduced here so dependent
//! crates can compile against them before any concrete backend lands.
//! Implementations (Wayland / X11 capture, Tesseract OCR) arrive in
//! Milestones 3–5.

use crate::error::PixelensError;
use crate::geometry::Rect;

/// RGBA pixel buffer produced by a capture backend.
///
/// Wrapping the raw bytes in a newtype keeps call sites from accidentally
/// confusing capture output with arbitrary `Vec<u8>` payloads. The format
/// is intentionally simple (RGBA, 4 bytes per pixel) — it's what the
/// Wayland `wlr-screencopy` and X11 `xcb_get_image` paths can both produce
/// with minimal conversion.
#[derive(Debug, Clone)]
pub struct CaptureImage {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub pixels: Vec<u8>,
}

impl CaptureImage {
    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty() || self.width == 0 || self.height == 0
    }
}

/// What a capture request returns: the selected region plus the pixel data.
///
/// The region is a generic bounding box, not a rectangle, so the
/// `CaptureProvider` trait does not assume v1's rectangle-only selection.
#[derive(Debug, Clone)]
pub struct RawCapture {
    pub region: Rect,
    pub image: CaptureImage,
}

/// Identifies a single capture session so the daemon can match cancel
/// signals to in-progress overlays (per PRD §"IPC").
#[derive(Debug, Clone)]
pub struct CaptureRequest {
    pub session_id: String,
}

/// Backend-agnostic screen capture.
///
/// Concrete implementations: `WaylandCaptureProvider` (M3), `X11CaptureProvider` (M4).
/// The daemon selects one at startup based on the display server detector (M2).
pub trait CaptureProvider: Send + Sync {
    /// Run an interactive capture (overlay appears, user selects, returns).
    fn capture(&self, request: &CaptureRequest) -> Result<RawCapture, PixelensError>;

    /// Cancel an in-flight session; the overlay must vanish cleanly with
    /// no on-screen artefacts.
    fn cancel(&self, session_id: &str);
}

/// Error type for the OCR layer. Distinct from `PixelensError` so the OCR
/// engine can describe its own failure modes without leaking them through
/// the umbrella enum.
#[derive(Debug, thiserror::Error)]
pub enum OcrError {
    #[error("tesseract binary not found")]
    EngineMissing,

    #[error("tesseract failed: {0}")]
    Engine(String),

    #[error("unsupported image format")]
    UnsupportedImage,
}

/// OCR backend. v1 implementation: `TesseractOcrEngine` (M5).
///
/// The trait hides engine internals — implementations own their warm-up
/// state and must be cheap to call repeatedly.
pub trait OcrEngine: Send + Sync {
    fn extract_text(&self, image: &CaptureImage) -> Result<String, OcrError>;
}
