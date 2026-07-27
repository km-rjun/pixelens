//! External-tool-backed region selection and screen capture.
//!
//! These are the v1-Wayland implementations. The PRD's long-term
//! `CaptureProvider` trait (in `pixelens-core`) is reserved for the
//! native wlr-layer-shell / wlr-screencopy path that lands in M3 —
//! slurp + grim are a different shape (separate processes, file-based
//! output) and shoehorning them into `CaptureProvider` would distort
//! that trait. See `docs/architecture.md` for the rationale.

use crate::Monitor;
use pixelens_core::{CaptureError, CaptureResult, Rect};
use std::path::Path;
use std::process::Stdio;

/// Geometry string in the `WxH+X+Y` form that `grim -g` expects.
///
/// We always emit at least 1x1 because grim rejects zero-sized regions.
pub fn format_geometry(r: Rect) -> String {
    let w = r.size.width.max(1);
    let h = r.size.height.max(1);
    let x = r.origin.x;
    let y = r.origin.y;
    format!("{w}x{h}+{x}+{y}")
}

/// Backend that prompts the user to select a rectangular region and
/// returns the geometry of the selection, or signals cancellation.
///
/// v1 implementation: [`SlurpSelector`], which shells out to `slurp`.
pub trait RegionSelector: Send + Sync {
    /// Block until the user makes a selection or cancels.
    fn select(&self) -> CaptureResult<Option<Rect>>;

    /// Select a region, optionally constrained to a specific monitor.
    ///
    /// Default implementation ignores the monitor and calls `select()`.
    /// Backends that support multi-monitor (e.g. slurp with `-o` output)
    /// can override this.
    fn select_with_monitor(&self, monitor: Option<&Monitor>) -> CaptureResult<Option<Rect>> {
        let _ = monitor;
        self.select()
    }
}

/// Backend that captures a rectangular region of the screen to a file.
///
/// v1 implementation: [`GrimCapturer`], which shells out to `grim`.
pub trait ScreenCapturer: Send + Sync {
    /// Capture `region` to `output_path`. The output format is determined
    /// by the file extension; `grim` will write PNG for `.png`, etc.
    fn capture(&self, region: Rect, output_path: &Path) -> CaptureResult<()>;

    /// Capture `region` to `output_path`, optionally constrained to a monitor.
    ///
    /// Default implementation ignores the monitor and calls `capture()`.
    /// Backends that support multi-monitor (e.g. grim with `-o` output)
    /// can override this.
    fn capture_with_monitor(
        &self,
        region: Rect,
        output_path: &Path,
        monitor: Option<&Monitor>,
    ) -> CaptureResult<()> {
        let _ = monitor;
        self.capture(region, output_path)
    }
}

/// `slurp`-backed region selector.
///
/// `slurp` prints geometry to stdout on success and exits with status 1
/// (empty stdout) on user cancel. We treat any non-zero exit with empty
/// stdout as cancellation rather than an error.
pub struct SlurpSelector {
    pub program: String,
}

impl Default for SlurpSelector {
    fn default() -> Self {
        Self {
            program: "slurp".to_string(),
        }
    }
}

impl SlurpSelector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_program(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl RegionSelector for SlurpSelector {
    fn select(&self) -> CaptureResult<Option<Rect>> {
        use std::process::Command;

        tracing::info!(program = %self.program, "invoking region selector");

        let output = Command::new(&self.program)
            .arg("-d")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CaptureError::ToolMissing(self.program.clone())
                } else {
                    CaptureError::Selector(e.to_string())
                }
            })?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() {
                tracing::info!("region selection cancelled by user");
                return Ok(None);
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CaptureError::Selector(format!(
                "slurp exited with status {}: {}",
                output.status,
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let geometry = stdout.trim();
        tracing::debug!(geometry, "slurp returned");

        let rect = parse_geometry(geometry).ok_or_else(|| {
            CaptureError::Selector(format!("could not parse slurp output: {geometry:?}"))
        })?;

        Ok(Some(rect))
    }

    fn select_with_monitor(&self, monitor: Option<&Monitor>) -> CaptureResult<Option<Rect>> {
        use std::process::Command;

        let mut cmd = Command::new(&self.program);
        cmd.arg("-d")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(m) = monitor {
            cmd.arg("-o").arg(&m.name);
            tracing::debug!(monitor = %m.name, "constraining slurp to monitor");
        }

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CaptureError::ToolMissing(self.program.clone())
            } else {
                CaptureError::Selector(e.to_string())
            }
        })?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() {
                tracing::info!("region selection cancelled by user");
                return Ok(None);
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CaptureError::Selector(format!(
                "slurp exited with status {}: {}",
                output.status,
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let geometry = stdout.trim();
        tracing::debug!(geometry, "slurp returned");

        let rect = parse_geometry(geometry).ok_or_else(|| {
            CaptureError::Selector(format!("could not parse slurp output: {geometry:?}"))
        })?;

        Ok(Some(rect))
    }
}

/// `grim`-backed screen capturer. Writes `output_path` (PNG if the path
/// ends in `.png`).
pub struct GrimCapturer {
    pub program: String,
}

impl Default for GrimCapturer {
    fn default() -> Self {
        Self {
            program: "grim".to_string(),
        }
    }
}

impl GrimCapturer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_program(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl ScreenCapturer for GrimCapturer {
    fn capture(&self, region: Rect, output_path: &Path) -> CaptureResult<()> {
        use std::process::Command;

        let geometry = format_geometry(region);
        tracing::info!(
            program = %self.program,
            geometry,
            output = %output_path.display(),
            "capturing region"
        );

        let status = Command::new(&self.program)
            .arg("-g")
            .arg(&geometry)
            .arg(output_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .status()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CaptureError::ToolMissing(self.program.clone())
                } else {
                    CaptureError::Capture(e.to_string())
                }
            })?;

        if !status.success() {
            return Err(CaptureError::Capture(format!(
                "grim exited with status {status}"
            )));
        }

        let meta = std::fs::metadata(output_path).map_err(|e| {
            CaptureError::Capture(format!(
                "grim reported success but output file is inaccessible: {e}"
            ))
        })?;

        if meta.len() == 0 {
            return Err(CaptureError::Capture(
                "grim wrote a 0-byte capture file".to_string(),
            ));
        }

        Ok(())
    }

    fn capture_with_monitor(
        &self,
        region: Rect,
        output_path: &Path,
        monitor: Option<&Monitor>,
    ) -> CaptureResult<()> {
        use std::process::Command;

        let geometry = format_geometry(region);
        let mut cmd = Command::new(&self.program);
        cmd.arg("-g").arg(&geometry).arg(output_path);

        if let Some(m) = monitor {
            cmd.arg("-o").arg(&m.name);
            tracing::debug!(monitor = %m.name, "constraining grim to monitor");
        }

        let status = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .status()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CaptureError::ToolMissing(self.program.clone())
                } else {
                    CaptureError::Capture(e.to_string())
                }
            })?;

        if !status.success() {
            return Err(CaptureError::Capture(format!(
                "grim exited with status {status}"
            )));
        }

        let meta = std::fs::metadata(output_path).map_err(|e| {
            CaptureError::Capture(format!(
                "grim reported success but output file is inaccessible: {e}"
            ))
        })?;

        if meta.len() == 0 {
            return Err(CaptureError::Capture(
                "grim wrote a 0-byte capture file".to_string(),
            ));
        }

        Ok(())
    }
}

/// Parse a geometry string in `WxH+X+Y` form.
pub fn parse_geometry(s: &str) -> Option<Rect> {
    let (size_part, offset_part) = s.split_once('+')?;
    let (w_str, h_str) = size_part.split_once('x')?;
    let width = w_str.parse().ok()?;
    let height = h_str.parse().ok()?;
    let offset_parts: Vec<&str> = offset_part.split('+').collect();
    if offset_parts.len() < 2 {
        return None;
    }
    let x = offset_parts[0].parse().ok()?;
    let y = offset_parts[1].parse().ok()?;
    Some(Rect::new(x, y, width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_geometry_round_trips() {
        let r = Rect::new(10, 20, 300, 200);
        let geom = format_geometry(r);
        let parsed = parse_geometry(&geom).unwrap();
        assert_eq!(r, parsed);
    }

    #[test]
    fn format_geometry_clamps_zero() {
        let r = Rect::new(0, 0, 0, 0);
        let geom = format_geometry(r);
        assert_eq!(geom, "1x1+0+0");
    }

    #[test]
    fn parse_geometry_with_offset() {
        let r = parse_geometry("1920x1080+1920+0").unwrap();
        assert_eq!(r.origin.x, 1920);
        assert_eq!(r.origin.y, 0);
        assert_eq!(r.size.width, 1920);
        assert_eq!(r.size.height, 1080);
    }

    #[test]
    fn parse_geometry_without_offset() {
        let r = parse_geometry("1920x1080+0+0").unwrap();
        assert_eq!(r.origin.x, 0);
        assert_eq!(r.origin.y, 0);
        assert_eq!(r.size.width, 1920);
        assert_eq!(r.size.height, 1080);
    }
}
