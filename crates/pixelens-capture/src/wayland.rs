use std::fs;
use std::process::Command;

use crate::ScreenCapture;
use pixelens_common::{CaptureError, CaptureRegion, CaptureResult};

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
    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() != 1 {
        return Err(CaptureError::ToolFailed(format!(
            "Unexpected slurp output: {}",
            output
        )));
    }

    let geometry = parts[0];
    let xy_wh: Vec<&str> = geometry.split('+').collect();
    if xy_wh.len() != 3 {
        return Err(CaptureError::ToolFailed(format!(
            "Invalid geometry format: {}",
            geometry
        )));
    }

    let xy: Vec<&str> = xy_wh[0].split('x').collect();
    if xy.len() != 2 {
        return Err(CaptureError::ToolFailed(format!(
            "Invalid XY format: {}",
            xy_wh[0]
        )));
    }

    let x: i32 = xy_wh[1]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid X coordinate: {}", xy_wh[1])))?;
    let y: i32 = xy_wh[2]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid Y coordinate: {}", xy_wh[2])))?;
    let width: u32 = xy[0]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid width: {}", xy[0])))?;
    let height: u32 = xy[1]
        .parse()
        .map_err(|_| CaptureError::ToolFailed(format!("Invalid height: {}", xy[1])))?;

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
    fn test_parse_slurp_output() {
        let output = "800x600+100+200";
        let region = parse_slurp_output(output).unwrap();
        assert_eq!(region.width, 800);
        assert_eq!(region.height, 600);
        assert_eq!(region.x, 100);
        assert_eq!(region.y, 200);
    }

    #[test]
    fn test_parse_slurp_invalid() {
        let result = parse_slurp_output("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_slurp_empty() {
        let result = parse_slurp_output("");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_available() {
        let capture = WaylandCapture;
        let result = capture.is_available();
        assert!(result, "WaylandCapture should be available");
    }
}
