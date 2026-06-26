pub mod wayland;

use crate::error::CaptureError;
use crate::types::{CaptureRegion, CaptureResult};

pub trait ScreenCapture {
    fn select_region(&self) -> Result<CaptureRegion, CaptureError>;
    fn capture(&self, region: &CaptureRegion) -> Result<CaptureResult, CaptureError>;
    fn is_available(&self) -> bool;
}

pub fn detect_backend() -> Result<Box<dyn ScreenCapture>, CaptureError> {
    let backend = wayland::WaylandCapture;
    if backend.is_available() {
        Ok(Box::new(backend))
    } else {
        Err(CaptureError::ToolNotFound(
            "No supported capture backend found (grim/slurp)".to_string(),
        ))
    }
}

pub fn check_tools() -> Vec<String> {
    let mut missing = Vec::new();
    if !tool_exists("slurp") {
        missing.push("slurp".to_string());
    }
    if !tool_exists("grim") {
        missing.push("grim".to_string());
    }
    missing
}

fn tool_exists(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_tools() {
        let missing = check_tools();
        assert!(missing.is_empty(), "Missing tools: {:?}", missing);
    }
}
