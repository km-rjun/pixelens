//! X11 capture backend (stub).
//!
//! M4 will implement this using XCB: a fullscreen InputOnly window with
//! a custom cursor handles region selection, and `xcb_get_image` captures
//! the selected region.

use pixelens_core::{CaptureProvider, CaptureRequest, CaptureResult, PixelensError};

pub struct X11CaptureProvider {
    _private: (),
}

impl X11CaptureProvider {
    pub fn new() -> Result<Self, PixelensError> {
        Ok(Self { _private: () })
    }
}

impl CaptureProvider for X11CaptureProvider {
    fn capture(&self, _request: &CaptureRequest) -> Result<CaptureResult, PixelensError> {
        Err(PixelensError::NotImplemented("X11CaptureProvider (M4)"))
    }

    fn cancel(&self, _session_id: &str) {
        // No-op until M4.
    }
}
