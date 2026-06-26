pub mod wayland;
pub mod x11;

use crate::error::CaptureError;
use crate::types::{CaptureRegion, CaptureResult};

pub trait ScreenCapture {
    fn select_region(&self) -> Result<CaptureRegion, CaptureError>;
    fn capture(&self, region: &CaptureRegion) -> Result<CaptureResult, CaptureError>;
    fn is_available(&self) -> bool;
}

pub fn detect_backend() -> Result<Box<dyn ScreenCapture>, CaptureError> {
    let wayland = wayland::WaylandCapture;
    if wayland.is_available() {
        return Ok(Box::new(wayland));
    }

    let x11 = x11::X11Capture;
    if x11.is_available() {
        return Ok(Box::new(x11));
    }

    Err(CaptureError::ToolNotFound(
        "No supported capture backend found (Wayland: grim/slurp, X11: scrot/slop)".to_string(),
    ))
}

pub fn check_tools() -> Vec<String> {
    let mut missing = Vec::new();

    let wayland_available = tool_exists("slurp") && tool_exists("grim");
    let x11_available = tool_exists("slop") && tool_exists("scrot");

    if !wayland_available && !x11_available {
        if !tool_exists("slurp") {
            missing.push("slurp".to_string());
        }
        if !tool_exists("grim") {
            missing.push("grim".to_string());
        }
        if !tool_exists("slop") {
            missing.push("slop".to_string());
        }
        if !tool_exists("scrot") {
            missing.push("scrot".to_string());
        }
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
    fn test_check_tools_runs() {
        let _missing = check_tools();
    }

    #[test]
    fn test_tool_exists_nonexistent() {
        assert!(!tool_exists("nonexistent_tool_xyz"));
    }

    #[test]
    #[ignore = "requires capture tools installed"]
    fn test_check_tools_all_present() {
        let missing = check_tools();
        assert!(missing.is_empty(), "Missing tools: {:?}", missing);
    }
}
