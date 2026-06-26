use std::fs;
use std::process::Command;

use crate::capture::ScreenCapture;
use crate::error::CaptureError;
use crate::types::{CaptureRegion, CaptureResult};

pub struct WaylandCapture;

impl ScreenCapture for WaylandCapture {
    fn select_region(&self) -> Result<CaptureRegion, CaptureError> {
        let output = Command::new("slurp")
            .output()
            .map_err(|e| CaptureError::ToolFailed(format!("Failed to run slurp: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("cancelled") || stderr.contains("Interrupted") {
                return Err(CaptureError::RegionCancelled);
            }
            return Err(CaptureError::ToolFailed(format!(
                "slurp failed: {}",
                stderr
            )));
        }

        let region_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if region_str.is_empty() {
            return Err(CaptureError::RegionCancelled);
        }

        parse_slurp_output(&region_str)
    }

    fn capture(&self, region: &CaptureRegion) -> Result<CaptureResult, CaptureError> {
        let geometry = region.to_grim_geometry();

        let output = Command::new("grim")
            .args(["-g", &geometry, "-"])
            .output()
            .map_err(|e| CaptureError::ToolFailed(format!("Failed to run grim: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CaptureError::ToolFailed(format!("grim failed: {}", stderr)));
        }

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "pixelens_{}.png",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        fs::write(&file_path, &output.stdout)
            .map_err(|e| CaptureError::ToolFailed(format!("Failed to write file: {}", e)))?;

        Ok(CaptureResult {
            image_path: file_path.to_string_lossy().to_string(),
            region: region.clone(),
        })
    }

    fn is_available(&self) -> bool {
        Command::new("slurp")
            .arg("-h")
            .output()
            .map(|o| o.status.success() || String::from_utf8_lossy(&o.stderr).contains("Usage"))
            .unwrap_or(false)
            && Command::new("grim")
                .arg("-h")
                .output()
                .map(|o| o.status.success() || String::from_utf8_lossy(&o.stderr).contains("Usage"))
                .unwrap_or(false)
    }
}

fn parse_slurp_output(output: &str) -> Result<CaptureRegion, CaptureError> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Err(CaptureError::ToolFailed("Empty slurp output".to_string()));
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(CaptureError::ToolFailed(format!(
            "Invalid slurp output format, expected 'x,y WxH', got: {}",
            trimmed
        )));
    }

    let xy_parts: Vec<&str> = parts[0].split(',').collect();
    if xy_parts.len() != 2 {
        return Err(CaptureError::ToolFailed(format!(
            "Invalid coordinate format, expected 'x,y', got: {}",
            parts[0]
        )));
    }

    let x: i32 = xy_parts[0]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid X coordinate: {}", xy_parts[0])))?;
    let y: i32 = xy_parts[1]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid Y coordinate: {}", xy_parts[1])))?;

    let wh_parts: Vec<&str> = parts[1].split('x').collect();
    if wh_parts.len() != 2 {
        return Err(CaptureError::ToolFailed(format!(
            "Invalid dimension format, expected 'WxH', got: {}",
            parts[1]
        )));
    }

    let width: u32 = wh_parts[0]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid width: {}", wh_parts[0])))?;
    let height: u32 = wh_parts[1]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid height: {}", wh_parts[1])))?;

    if width == 0 || height == 0 {
        return Err(CaptureError::ToolFailed(format!(
            "Invalid dimensions: {}x{}, width and height must be > 0",
            width, height
        )));
    }

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
    fn test_parse_standard_format() {
        let region = parse_slurp_output("424,220 615x227").unwrap();
        assert_eq!(region.x, 424);
        assert_eq!(region.y, 220);
        assert_eq!(region.width, 615);
        assert_eq!(region.height, 227);
    }

    #[test]
    fn test_parse_negative_coords() {
        let region = parse_slurp_output("-1920,0 1920x1080").unwrap();
        assert_eq!(region.x, -1920);
        assert_eq!(region.y, 0);
        assert_eq!(region.width, 1920);
        assert_eq!(region.height, 1080);
    }

    #[test]
    fn test_parse_trailing_newline() {
        let region = parse_slurp_output("424,220 615x227\n").unwrap();
        assert_eq!(region.x, 424);
        assert_eq!(region.y, 220);
        assert_eq!(region.width, 615);
        assert_eq!(region.height, 227);
    }

    #[test]
    fn test_parse_trailing_whitespace() {
        let region = parse_slurp_output("424,220 615x227  ").unwrap();
        assert_eq!(region.x, 424);
        assert_eq!(region.y, 220);
        assert_eq!(region.width, 615);
        assert_eq!(region.height, 227);
    }

    #[test]
    fn test_parse_malformed_too_few_parts() {
        let result = parse_slurp_output("424,220");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_malformed_no_comma() {
        let result = parse_slurp_output("424 220 615x227");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_malformed_no_x() {
        let result = parse_slurp_output("424,220 615-227");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_zero_width() {
        let result = parse_slurp_output("0,0 0x100");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_zero_height() {
        let result = parse_slurp_output("0,0 100x0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty() {
        let result = parse_slurp_output("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let result = parse_slurp_output("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_x() {
        let result = parse_slurp_output("abc,220 615x227");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_width() {
        let result = parse_slurp_output("424,220 xyzx227");
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "requires Wayland session with grim and slurp"]
    fn test_is_available() {
        let capture = WaylandCapture;
        let result = capture.is_available();
        assert!(result, "WaylandCapture should be available");
    }
}
