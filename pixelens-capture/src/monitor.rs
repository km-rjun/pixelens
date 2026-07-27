//! Monitor detection for multi-display setups (UM6).
//!
//! Detects which output the cursor is currently on, so `slurp`/`grim`
//! can be scoped to that single output instead of the full virtual
//! desktop. Falls back gracefully if detection is unavailable.

use crate::DisplayServer;
use pixelens_core::{PixelensError, PixelensResult, Point, Rect, Size};
use std::process::Command;

/// A single physical output / monitor.
#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    /// Output name (e.g., "eDP-1", "HDMI-A-1", "DP-2").
    pub name: String,
    /// Screen-space geometry of this output in pixels.
    pub geometry: Rect,
    /// Scale factor (e.g., 1.0, 2.0 for HiDPI).
    pub scale: f32,
}

/// Detect the output containing the cursor.
///
/// Wayland: uses `hyprctl cursorpos` + `hyprctl monitors` (Hyprland)
///          or `wlr-randr --json` + seat (generic wlroots)
/// X11:     uses `xrandr --query` + `xdotool getmouselocation`
///
/// Returns `Ok(None)` if detection runs but cursor isn't on any output,
/// or if the compositor doesn't support the required protocol.
pub fn detect_active_monitor(display: DisplayServer) -> PixelensResult<Option<Monitor>> {
    match display {
        DisplayServer::Wayland => detect_wayland_monitor(),
        DisplayServer::X11 => detect_x11_monitor(),
    }
}

fn detect_wayland_monitor() -> PixelensResult<Option<Monitor>> {
    // Try Hyprland first (common, well-supported)
    if let Ok(Some(m)) = try_hyprland() {
        return Ok(Some(m));
    }
    // Fallback: wlr-randr (generic wlroots)
    if let Ok(Some(m)) = try_wlr_randr() {
        return Ok(Some(m));
    }
    Ok(None)
}

fn try_hyprland() -> PixelensResult<Option<Monitor>> {
    // Get cursor position
    let cursor_out = Command::new("hyprctl").args(["cursorpos"]).output();
    let cursor_out = match cursor_out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return Ok(None),
    };
    // hyprctl cursorpos returns "x, y"
    let (cx, cy) = parse_xy(&cursor_out)?;

    // Get monitors
    let mon_out = Command::new("hyprctl").args(["monitors", "-j"]).output();
    let mon_out = match mon_out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return Ok(None),
    };

    // Parse JSON array of monitors
    let monitors: serde_json::Value = serde_json::from_str(&mon_out)
        .map_err(|e| PixelensError::Config(format!("hyprctl monitors JSON parse: {e}")))?;

    let arr = monitors
        .as_array()
        .ok_or_else(|| PixelensError::Config("hyprctl monitors: expected array".to_string()))?;

    for mon in arr {
        let name = mon
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let x = mon.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let y = mon.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let w = mon.get("width").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let h = mon.get("height").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let scale = mon.get("scale").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;

        if cx >= x && cx < x + w && cy >= y && cy < y + h {
            return Ok(Some(Monitor {
                name,
                geometry: Rect {
                    origin: Point::new(x, y),
                    size: Size::new(w as u32, h as u32),
                },
                scale,
            }));
        }
    }
    Ok(None)
}

fn try_wlr_randr() -> PixelensResult<Option<Monitor>> {
    // wlr-randr --json outputs array of outputs with position/size
    let out = Command::new("wlr-randr").arg("--json").output();
    let out = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return Ok(None),
    };

    // For wlr-randr without seat integration, we can't reliably detect
    // cursor position. Return first output as fallback for single-monitor.
    let out_str = out;
    let outputs: serde_json::Value = serde_json::from_str(&out_str)
        .map_err(|e| PixelensError::Config(format!("wlr-randr JSON parse: {e}")))?;

    let arr = outputs
        .as_array()
        .ok_or_else(|| PixelensError::Config("wlr-randr: expected array".to_string()))?;

    if let Some(first) = arr.first() {
        let name = first
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let x = first.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let y = first.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let w = first.get("width").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let h = first.get("height").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let scale = first.get("scale").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;

        return Ok(Some(Monitor {
            name,
            geometry: Rect {
                origin: Point::new(x, y),
                size: Size::new(w as u32, h as u32),
            },
            scale,
        }));
    }
    Ok(None)
}

fn detect_x11_monitor() -> PixelensResult<Option<Monitor>> {
    // Get mouse position
    let pos_out = Command::new("xdotool").args(["getmouselocation"]).output();
    let pos_out = match pos_out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return Ok(None),
    };
    // xdotool getmouselocation returns "x:123 y:456 screen:0 window:123456"
    let (cx, cy) = parse_xdotool_pos(&pos_out)?;

    // Get xrandr output
    let randr_out = Command::new("xrandr").arg("--query").output();
    let randr_out = match randr_out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return Ok(None),
    };

    // Parse xrandr --query for connected outputs with position
    for line in randr_out.lines() {
        let line = line.trim();
        if line.contains(" connected") {
            // Format: "HDMI-1 connected primary 1920x1080+0+0 (normal left inverted ...) 597mm x 336mm"
            if let Some(mon) = parse_xrandr_line(line) {
                if cx >= mon.geometry.origin.x
                    && cx < mon.geometry.origin.x + mon.geometry.size.width as i32
                    && cy >= mon.geometry.origin.y
                    && cy < mon.geometry.origin.y + mon.geometry.size.height as i32
                {
                    return Ok(Some(mon));
                }
            }
        }
    }
    Ok(None)
}

fn parse_xy(s: &str) -> PixelensResult<(i32, i32)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(PixelensError::Config(format!(
            "invalid cursorpos format: {}",
            s
        )));
    }
    let x = parts[0]
        .trim()
        .parse()
        .map_err(|e| PixelensError::Config(format!("parse x: {e}")))?;
    let y = parts[1]
        .trim()
        .parse()
        .map_err(|e| PixelensError::Config(format!("parse y: {e}")))?;
    Ok((x, y))
}

fn parse_xdotool_pos(s: &str) -> PixelensResult<(i32, i32)> {
    // "x:123 y:456 screen:0 window:123456"
    let mut x = 0i32;
    let mut y = 0i32;
    for part in s.split_whitespace() {
        if let Some(stripped) = part.strip_prefix("x:") {
            x = stripped.parse().unwrap_or(0);
        } else if let Some(stripped) = part.strip_prefix("y:") {
            y = stripped.parse().unwrap_or(0);
        }
    }
    Ok((x, y))
}

fn parse_xrandr_line(line: &str) -> Option<Monitor> {
    // "HDMI-1 connected primary 1920x1080+0+0 (normal left inverted ...) 597mm x 336mm"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 || !parts[1].contains("connected") {
        return None;
    }
    let name = parts[0].to_string();
    // Find the geometry part (e.g., "1920x1080+0+0")
    let geom = parts.iter().find(|p| p.contains("x") && p.contains("+"))?;
    let (size, offset) = geom.split_once("+")?;
    let (w_str, h_str) = size.split_once("x")?;
    let width = w_str.parse().ok()?;
    let height = h_str.parse().ok()?;
    let offset_parts: Vec<&str> = offset.split("+").collect();
    if offset_parts.len() != 2 {
        return None;
    }
    let x = offset_parts[0].parse().ok()?;
    let y = offset_parts[1].parse().ok()?;
    Some(Monitor {
        name,
        geometry: Rect::new(x, y, width, height),
        scale: 1.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_xdotool_pos_works() {
        let (x, y) = parse_xdotool_pos("x:123 y:456 screen:0 window:123456").unwrap();
        assert_eq!(x, 123);
        assert_eq!(y, 456);
    }

    #[test]
    fn parse_xrandr_line_works() {
        let line =
            "HDMI-1 connected primary 1920x1080+0+0 (normal left inverted ...) 597mm x 336mm";
        let mon = parse_xrandr_line(line).unwrap();
        assert_eq!(mon.name, "HDMI-1");
        assert_eq!(mon.geometry.origin.x, 0);
        assert_eq!(mon.geometry.origin.y, 0);
        assert_eq!(mon.geometry.size.width, 1920);
        assert_eq!(mon.geometry.size.height, 1080);
    }
}
