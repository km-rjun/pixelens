use std::process::Command;

use crate::capture::ScreenCapture;
use crate::error::CaptureError;
use crate::types::{CaptureRegion, CaptureResult};

pub struct X11Capture;

impl ScreenCapture for X11Capture {
    fn select_region(&self) -> Result<CaptureRegion, CaptureError> {
        let output = Command::new("slop")
            .args(["-f", "%x %y %w %h"])
            .output()
            .map_err(|e| CaptureError::ToolFailed(format!("Failed to run slop: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("cancelled") || stderr.contains("Interrupted") {
                return Err(CaptureError::RegionCancelled);
            }
            return Err(CaptureError::ToolFailed(format!("slop failed: {}", stderr)));
        }

        let region_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if region_str.is_empty() {
            return Err(CaptureError::RegionCancelled);
        }

        parse_slop_output(&region_str)
    }

    fn capture(&self, region: &CaptureRegion) -> Result<CaptureResult, CaptureError> {
        let geometry = format!(
            "{}x{}+{}+{}",
            region.width, region.height, region.x, region.y
        );

        let output = Command::new("scrot")
            .args(["-a", &geometry, "-o", "/tmp/pixelens_x11.png"])
            .output()
            .map_err(|e| CaptureError::ToolFailed(format!("Failed to run scrot: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CaptureError::ToolFailed(format!(
                "scrot failed: {}",
                stderr
            )));
        }

        let file_path = "/tmp/pixelens_x11.png".to_string();

        Ok(CaptureResult {
            image_path: file_path,
            region: region.clone(),
        })
    }

    fn is_available(&self) -> bool {
        Command::new("slop")
            .arg("-h")
            .output()
            .map(|o| o.status.success() || String::from_utf8_lossy(&o.stderr).contains("Usage"))
            .unwrap_or(false)
            && Command::new("scrot")
                .arg("-h")
                .output()
                .map(|o| o.status.success() || String::from_utf8_lossy(&o.stderr).contains("Usage"))
                .unwrap_or(false)
    }
}

fn parse_slop_output(output: &str) -> Result<CaptureRegion, CaptureError> {
    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() != 4 {
        return Err(CaptureError::ToolFailed(format!(
            "Unexpected slop output: {}",
            output
        )));
    }

    let x: i32 = parts[0]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid X coordinate: {}", parts[0])))?;
    let y: i32 = parts[1]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid Y coordinate: {}", parts[1])))?;
    let width: u32 = parts[2]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid width: {}", parts[2])))?;
    let height: u32 = parts[3]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid height: {}", parts[3])))?;

    Ok(CaptureRegion {
        x,
        y,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slop_output() {
        let output = "100 200 800 600";
        let region = parse_slop_output(output).unwrap();
        assert_eq!(region.x, 100);
        assert_eq!(region.y, 200);
        assert_eq!(region.width, 800);
        assert_eq!(region.height, 600);
    }

    #[test]
    fn test_parse_slop_invalid() {
        let result = parse_slop_output("invalid");
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "requires slop and scrot installed"]
    fn test_is_available() {
        let capture = X11Capture;
        let result = capture.is_available();
        assert!(result, "X11Capture should be available");
    }
}
