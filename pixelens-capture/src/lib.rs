//! Display server detection and capture backends.
//!
//! M1 brought the detector and stub Wayland/X11 `CaptureProvider`
//! implementations. M6 introduces a separate, leaner path for the
//! v1-Wayland `slurp` + `grim` workflow in [`slurp_grim`]: the
//! slurp/grim process model is so different from the long-term
//! in-process wlr-screencopy path that shoehorning it into
//! `CaptureProvider` would distort the trait. See the module
//! documentation in `slurp_grim.rs` for the rationale.

pub mod detector;
pub mod pipeline;
pub mod slurp_grim;
pub mod wayland;
pub mod which;
pub mod x11;

pub use detector::{detect_display_server, DisplayServer};
pub use pipeline::{GrabError, GrabErrorKind, GrabOutcome, GrabPipeline};
pub use slurp_grim::{
    format_geometry, parse_geometry, GrimCapturer, RegionSelector, ScreenCapturer, SlurpSelector,
};
pub use which::{which, WhichError};

use pixelens_core::{CaptureProvider, CaptureRequest, PixelensError, RawCapture};
use std::sync::Arc;

/// Concrete capture provider selected at daemon startup (long-term path).
///
/// Wraps whichever `CaptureProvider` matches the detected display server.
/// The daemon builds one of these once and never branches on display
/// server type again (PRD §"Display Server Detection"). The v1-Wayland
/// slurp/grim path lives in `slurp_grim` and is selected independently.
pub enum CaptureBackend {
    Wayland(Arc<wayland::WaylandCaptureProvider>),
    X11(Arc<x11::X11CaptureProvider>),
}

impl CaptureProvider for CaptureBackend {
    fn capture(&self, request: &CaptureRequest) -> Result<RawCapture, PixelensError> {
        match self {
            CaptureBackend::Wayland(p) => p.capture(request),
            CaptureBackend::X11(p) => p.capture(request),
        }
    }

    fn cancel(&self, session_id: &str) {
        match self {
            CaptureBackend::Wayland(p) => p.cancel(session_id),
            CaptureBackend::X11(p) => p.cancel(session_id),
        }
    }
}
