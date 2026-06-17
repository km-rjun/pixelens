//! Display server detection and capture backends.
//!
//! M1: the detector is implemented (it has no dependencies beyond
//! `std::env`) and the `CaptureProvider` trait is satisfied by stub
//! Wayland/X11 implementations. The real Wayland (layer-shell +
//! wlr-screencopy) and X11 (XCB) backends arrive in M3 and M4.

pub mod detector;
pub mod wayland;
pub mod x11;

pub use detector::{detect_display_server, DisplayServer};

use pixelens_core::{CaptureProvider, CaptureRequest, CaptureResult, PixelensError};
use std::sync::Arc;

/// Concrete capture provider selected at daemon startup.
///
/// Wraps whichever `CaptureProvider` matches the detected display server.
/// The daemon builds one of these once and never branches on display
/// server type again (PRD §"Display Server Detection").
pub enum CaptureBackend {
    Wayland(Arc<wayland::WaylandCaptureProvider>),
    X11(Arc<x11::X11CaptureProvider>),
}

impl CaptureProvider for CaptureBackend {
    fn capture(&self, request: &CaptureRequest) -> Result<CaptureResult, PixelensError> {
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
