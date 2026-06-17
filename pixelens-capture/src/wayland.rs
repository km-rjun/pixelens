//! Wayland capture backend (stub).
//!
//! M3 will implement this using `zwlr-layer-shell-v1` for the overlay
//! and `zwlr-screencopy-manager-v1` for capture, with `xdg-desktop-portal`
//! as the fallback for compositors that don't expose wlr protocols
//! (GNOME, KDE). The v1-Wayland slurp/grim path lives in `slurp_grim`.

use pixelens_core::{CaptureProvider, CaptureRequest, PixelensError, RawCapture};

pub struct WaylandCaptureProvider {
    _private: (),
}

impl WaylandCaptureProvider {
    pub fn new() -> Result<Self, PixelensError> {
        Ok(Self { _private: () })
    }
}

impl CaptureProvider for WaylandCaptureProvider {
    fn capture(&self, _request: &CaptureRequest) -> Result<RawCapture, PixelensError> {
        Err(PixelensError::NotImplemented("WaylandCaptureProvider (M3)"))
    }

    fn cancel(&self, _session_id: &str) {
        // No-op until M3.
    }
}
