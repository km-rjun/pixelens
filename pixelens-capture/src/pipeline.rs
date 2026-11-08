//! End-to-end capture pipeline: `slurp` -> `grim` -> temp file.
//!
//! This is the v1-Wayland grab path. The pipeline:
//!  1. Verifies `slurp` and `grim` are on `$PATH`.
//!  2. Asks `slurp` to pick a region (blocks until the user picks or
//!     cancels; cancel returns `Ok(None)`).
//!  3. Asks `grim` to write the selected region to a fresh temp file.
//!  4. Returns the temp file path and the selected geometry.
//!
//! Cancel / failure / missing-tool are all surfaced as
//! [`GrabOutcome`] values, not raw subprocess errors, so the daemon's
//! IPC layer can map them to response statuses cleanly.

use crate::monitor::detect_active_monitor;
#[cfg(windows)]
use crate::monitor::detect_active_monitor_windows;
#[cfg(unix)]
use crate::slurp_grim::{GrimCapturer, SlurpSelector};
use crate::slurp_grim::{RegionSelector, ScreenCapturer};
use crate::{CaptureError, Rect};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Result of a full `slurp + grim` capture run.
#[derive(Debug, Clone)]
pub enum GrabOutcome {
    /// User selected a region and the screenshot was written.
    Captured {
        path: PathBuf,
        region: Rect,
        bytes: u64,
    },
    /// User pressed Escape in slurp.
    Cancelled,
}

/// Why a grab failed. Distinct from `Cancelled` so the daemon can show
/// a useful error notification vs. a silent "cancelled" message.
#[derive(Debug, Clone)]
pub struct GrabError {
    pub kind: GrabErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrabErrorKind {
    /// `slurp` or `grim` not installed.
    MissingTool,
    /// Subprocess ran but exited non-zero with stderr.
    Subprocess,
    /// `grim` claimed success but the file is missing or zero bytes.
    Output,
}

impl std::fmt::Display for GrabError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for GrabError {}

/// Pipeline configuration. Defaults to the standard `slurp` + `grim`
/// binaries on `$PATH`; tests can override with stub paths.
pub struct GrabPipeline {
    selector: Box<dyn RegionSelector>,
    capturer: Box<dyn ScreenCapturer>,
    /// Caller-provided output directory. Defaults to the system temp
    /// dir if `None`.
    output_dir: Option<PathBuf>,
}

impl GrabPipeline {
    /// Standard pipeline: region selection + capture.
    ///
    /// On Unix this is `slurp` + `grim` (v1-Wayland). On Windows it is the
    /// WinRT `GraphicsCapturePicker` (Snipping-Tool-class experience bound to
    /// the `Win+Shift+S` hotkey). Both arms do an upfront dependency probe so
    /// the caller gets one clear message at startup.
    pub fn new() -> Result<Self, GrabError> {
        #[cfg(windows)]
        {
            crate::windows::region_selector(); // type-check the path is wired
            Self::with_selector_and_capturer(
                crate::windows::region_selector(),
                crate::windows::screen_capturer(),
            )
        }
        #[cfg(unix)]
        {
            check_dependency("slurp")?;
            check_dependency("grim")?;
            Self::with_selector_and_capturer(
                Box::new(SlurpSelector::new()),
                Box::new(GrimCapturer::new()),
            )
        }
    }

    /// Custom selector + capturer (used by tests). Skips the upfront
    /// dependency probe; the caller is responsible for ensuring the
    /// underlying tools exist.
    #[cfg(unix)]
    pub fn with_selector_and_capturer(
        selector: Box<dyn RegionSelector>,
        capturer: Box<dyn ScreenCapturer>,
    ) -> Result<Self, GrabError> {
        Ok(Self {
            selector,
            capturer,
            output_dir: None,
        })
    }

    #[cfg(windows)]
    pub fn with_selector_and_capturer(
        selector: Box<dyn RegionSelector>,
        capturer: Box<dyn ScreenCapturer>,
    ) -> Result<Self, GrabError> {
        Ok(Self {
            selector,
            capturer,
            output_dir: None,
        })
    }

    /// Set the directory in which capture files are written. Defaults
    /// to the system temp dir.
    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = Some(dir);
        self
    }

    /// Run the full pipeline with multi-monitor support (UM6).
    ///
    /// 1. Detect active monitor (Wayland: hyprctl/wlr-randr; X11: xrandr+xdotool).
    /// 2. If a monitor is detected, constrain slurp/grim to that output.
    /// 2. Ask `slurp` to select a region (blocks until user picks or cancels).
    /// 3. Ask `grim` to write the selected region to a temp file.
    /// 4. Returns the temp file path and selected geometry.
    pub fn run(&self) -> Result<GrabOutcome, GrabError> {
        // Step 0: Detect active monitor for multi-monitor support (UM6)
        #[cfg(unix)]
        let monitor = detect_active_monitor().map_err(|e| GrabError {
            kind: GrabErrorKind::Subprocess,
            message: format!("monitor detection failed: {e}"),
        })?;

        #[cfg(windows)]
        let monitor = detect_active_monitor_windows().map_err(|e| GrabError {
            kind: GrabErrorKind::Subprocess,
            message: format!("monitor detection failed: {e}"),
        })?;

        // 1. region selection (with optional monitor constraint)
        let region = self
            .selector
            .select_with_monitor(monitor.as_ref())
            .map_err(|e| grab_error_from_capture(&e, "selector"))?;
        let region = match region {
            Some(r) => r,
            None => return Ok(GrabOutcome::Cancelled),
        };

        // 2. build output path
        let path = self.allocate_output_path();

        // 3. capture (with optional monitor constraint for grim)
        self.capturer
            .capture_with_monitor(region, &path, monitor.as_ref())
            .map_err(|e| grab_error_from_capture(&e, "capturer"))?;

        // 4. verify file size (grim normally errors on its own, but
        //    a 0-byte file means something went very wrong).
        let bytes = std::fs::metadata(&path)
            .map_err(|e| GrabError {
                kind: GrabErrorKind::Output,
                message: format!("could not stat capture output {}: {e}", path.display()),
            })?
            .len();

        if bytes == 0 {
            return Err(GrabError {
                kind: GrabErrorKind::Output,
                message: "grim wrote a 0-byte capture file".to_string(),
            });
        }

        tracing::info!(
            path = %path.display(),
            region_x = region.origin.x,
            region_y = region.origin.y,
            region_w = region.size.width,
            region_h = region.size.height,
            bytes,
            "capture pipeline complete"
        );

        Ok(GrabOutcome::Captured {
            path,
            region,
            bytes,
        })
    }

    fn allocate_output_path(&self) -> PathBuf {
        let dir = match self.output_dir.clone() {
            Some(d) => d,
            None => std::env::temp_dir(),
        };
        let dir = dir.join("pixelens");
        // Best-effort create; if it fails, grim will surface a write
        // error of its own and we report it as a subprocess error.
        let _ = std::fs::create_dir_all(&dir);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        dir.join(format!("capture-{stamp}.png"))
    }
}

impl Default for GrabPipeline {
    fn default() -> Self {
        Self::new().expect("slurp/grim dependency check failed at construction")
    }
}

#[cfg(unix)]
fn check_dependency(tool: &str) -> Result<(), GrabError> {
    if crate::which(tool).is_err() {
        Err(GrabError {
            kind: GrabErrorKind::MissingTool,
            message: format!(
                "{} is not installed (or not on $PATH). {}",
                tool,
                install_hint(tool)
            ),
        })
    } else {
        Ok(())
    }
}

fn grab_error_from_capture(e: &CaptureError, tool: &str) -> GrabError {
    match e {
        CaptureError::ToolMissing(name, hint) => GrabError {
            kind: GrabErrorKind::MissingTool,
            message: format!("{} is not installed (or not on $PATH). {}", name, hint),
        },
        CaptureError::Selector(msg) => GrabError {
            kind: GrabErrorKind::Subprocess,
            message: format!("{} selector failed: {}", tool, msg),
        },
        CaptureError::Capture(msg) => GrabError {
            kind: GrabErrorKind::Subprocess,
            message: format!("{} capture failed: {}", tool, msg),
        },
        CaptureError::Io(err) => GrabError {
            kind: GrabErrorKind::Subprocess,
            message: format!("{} io error: {}", tool, err),
        },
    }
}

pub fn install_hint(tool: &str) -> &'static str {
    match tool {
        "slurp" => {
            "Install via your package manager (e.g. `apt install slurp` or `pacman -S slurp`)."
        }
        "grim" => "Install via your package manager (e.g. `apt install grim` or `pacman -S grim`).",
        "xdotool" => {
            "Install via your package manager (e.g. `apt install xdotool` or `pacman -S xdotool`)."
        }
        "hyprctl" => "Requires Hyprland compositor.",
        "wlr-randr" => "Install wlr-randr (`cargo install wlr-randr` or distro package).",
        "xrandr" => "Usually pre-installed with X11.",
        _ => "Install via your package manager.",
    }
}
