pub mod wayland;

use crate::error::CaptureError;
use crate::types::{CaptureRegion, CaptureResult};

pub trait ScreenCapture {
    fn select_region(&self) -> Result<CaptureRegion, CaptureError>;
    fn capture(&self, region: &CaptureRegion) -> Result<CaptureResult, CaptureError>;
}

pub fn detect_backend() -> Box<dyn ScreenCapture> {
    Box::new(wayland::WaylandCapture)
}
