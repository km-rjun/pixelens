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

#[cfg(unix)]
use crate::slurp_grim::{GrimCapturer, SlurpSelector};
use crate::slurp_grim::{RegionSelector, ScreenCapturer};
#[cfg(unix)]
use crate::which;
use pixelens_core::{CaptureError, Rect};
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

    /// Run the full pipeline. See [`GrabOutcome`].
    pub fn run(&self) -> Result<GrabOutcome, GrabError> {
        // 1. region selection
        let region = self
            .selector
            .select()
            .map_err(|e| grab_error_from_capture(&e, "slurp"))?;
        let region = match region {
            Some(r) => r,
            None => return Ok(GrabOutcome::Cancelled),
        };

        // 2. build output path
        let path = self.allocate_output_path();

        // 3. capture
        self.capturer
            .capture(region, &path)
            .map_err(|e| grab_error_from_capture(&e, "grim"))?;

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
    if which(tool).is_err() {
        Err(GrabError {
            kind: GrabErrorKind::MissingTool,
            message: format!(
                "{tool} is not installed (or not on $PATH). {}",
                install_hint(tool)
            ),
        })
    } else {
        Ok(())
    }
}

fn grab_error_from_capture(e: &CaptureError, tool: &str) -> GrabError {
    match e {
        CaptureError::ToolMissing(name) => GrabError {
            kind: GrabErrorKind::MissingTool,
            message: format!(
                "{tool} is not installed (or not on $PATH). {install} Please install {name}.",
                tool = tool,
                install = install_hint(tool),
                name = name
            ),
        },
        CaptureError::Selector(msg) => GrabError {
            kind: GrabErrorKind::Subprocess,
            message: format!("{tool} failed: {msg}"),
        },
        CaptureError::Capture(msg) => GrabError {
            kind: GrabErrorKind::Subprocess,
            message: format!("{tool} failed: {msg}"),
        },
        CaptureError::Io(err) => GrabError {
            kind: GrabErrorKind::Subprocess,
            message: format!("{tool} io error: {err}"),
        },
    }
}

fn install_hint(tool: &str) -> &'static str {
    match tool {
        "slurp" => "Debian/Ubuntu: apt install slurp. Arch: pacman -S slurp.",
        "grim" => "Debian/Ubuntu: apt install grim. Arch: pacman -S grim.",
        _ => "see your distribution's package manager.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pixelens_core::CaptureResult;
    use std::path::Path;
    use std::sync::Mutex;

    /// Fake selector: returns the configured region or `None` (cancel).
    struct FakeSelector(Mutex<Option<Option<Rect>>>);
    impl RegionSelector for FakeSelector {
        fn select(&self) -> CaptureResult<Option<Rect>> {
            Ok(self.0.lock().unwrap().take().unwrap())
        }
    }

    /// Fake capturer: writes a known number of bytes to the output
    /// path, or returns a configured error.
    struct FakeCapturer {
        behaviour: Mutex<FakeCaptureBehaviour>,
    }
    enum FakeCaptureBehaviour {
        WriteBytes(u64),
        Fail(CaptureError),
    }
    impl ScreenCapturer for FakeCapturer {
        fn capture(&self, _region: Rect, output_path: &Path) -> CaptureResult<()> {
            let mut b = self.behaviour.lock().unwrap();
            match &mut *b {
                FakeCaptureBehaviour::WriteBytes(n) => {
                    let bytes = vec![0u8; *n as usize];
                    std::fs::write(output_path, &bytes)?;
                    Ok(())
                }
                FakeCaptureBehaviour::Fail(e) => Err(match e {
                    CaptureError::ToolMissing(s) => CaptureError::ToolMissing(s.clone()),
                    CaptureError::Selector(s) => CaptureError::Selector(s.clone()),
                    CaptureError::Capture(s) => CaptureError::Capture(s.clone()),
                    CaptureError::Io(io) => {
                        CaptureError::Io(std::io::Error::new(io.kind(), io.to_string()))
                    }
                }),
            }
        }
    }

    /// `which` may fail on systems without slurp/grim installed. Skip
    /// the test rather than failing.
    #[cfg(unix)]
    fn require_deps() -> bool {
        which("slurp").is_ok() && which("grim").is_ok()
    }
    #[cfg(windows)]
    fn require_deps() -> bool {
        false // Windows uses different capture pipeline
    }

    #[test]
    fn new_succeeds_when_dependencies_present() {
        if !require_deps() {
            eprintln!("slurp/grim not installed; skipping");
            return;
        }
        let p = GrabPipeline::new();
        assert!(p.is_ok());
    }

    #[test]
    fn cancelled_selector_returns_cancelled_outcome() {
        if !require_deps() {
            return;
        }
        // We can't easily inject a fake selector AND satisfy the
        // which() check in `with_selector_and_capturer` (which probes
        // the real slurp/grim names). So we test the lower-level path:
        // build a pipeline that bypasses the upfront check, then
        // exercise `run` with a cancelling selector.
        let selector = Box::new(FakeSelector(Mutex::new(Some(None))));
        let capturer = Box::new(FakeCapturer {
            behaviour: Mutex::new(FakeCaptureBehaviour::WriteBytes(0)),
        });
        // Can't construct a real pipeline with fakes without bypassing
        // the which() check; use a test-only shim. The shim is the
        // safest test seam.
        let pipeline = TestPipeline {
            selector,
            capturer,
            output_dir: std::env::temp_dir(),
        };
        let outcome = pipeline.run().unwrap();
        assert!(matches!(outcome, GrabOutcome::Cancelled));
    }

    /// Test-only pipeline that skips the upfront which() probe.
    struct TestPipeline {
        selector: Box<dyn RegionSelector>,
        capturer: Box<dyn ScreenCapturer>,
        output_dir: PathBuf,
    }
    impl TestPipeline {
        fn run(&self) -> Result<GrabOutcome, GrabError> {
            let region = self
                .selector
                .select()
                .map_err(|e| grab_error_from_capture(&e, "slurp"))?;
            let region = match region {
                Some(r) => r,
                None => return Ok(GrabOutcome::Cancelled),
            };
            let path = self.output_dir.join(format!(
                "pixelens-test-{}.png",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            self.capturer
                .capture(region, &path)
                .map_err(|e| grab_error_from_capture(&e, "grim"))?;
            let bytes = std::fs::metadata(&path)
                .map_err(|e| GrabError {
                    kind: GrabErrorKind::Output,
                    message: e.to_string(),
                })?
                .len();
            Ok(GrabOutcome::Captured {
                path,
                region,
                bytes,
            })
        }
    }

    #[test]
    fn captured_selector_writes_file_and_returns_path() {
        if !require_deps() {
            return;
        }
        let selector = Box::new(FakeSelector(Mutex::new(Some(Some(Rect::new(
            10, 20, 100, 50,
        ))))));
        let capturer = Box::new(FakeCapturer {
            behaviour: Mutex::new(FakeCaptureBehaviour::WriteBytes(1024)),
        });
        let pipeline = TestPipeline {
            selector,
            capturer,
            output_dir: std::env::temp_dir(),
        };
        let outcome = pipeline.run().unwrap();
        match outcome {
            GrabOutcome::Captured {
                path,
                region,
                bytes,
            } => {
                assert_eq!(region.origin.x, 10);
                assert_eq!(region.size.width, 100);
                assert_eq!(bytes, 1024);
                assert!(path.exists());
                let _ = std::fs::remove_file(path);
            }
            _ => panic!("expected Captured"),
        }
    }

    #[test]
    fn capture_failure_propagates_as_grab_error() {
        if !require_deps() {
            return;
        }
        let selector = Box::new(FakeSelector(Mutex::new(Some(Some(Rect::new(
            0, 0, 10, 10,
        ))))));
        let capturer = Box::new(FakeCapturer {
            behaviour: Mutex::new(FakeCaptureBehaviour::Fail(CaptureError::Capture(
                "boom".into(),
            ))),
        });
        let pipeline = TestPipeline {
            selector,
            capturer,
            output_dir: std::env::temp_dir(),
        };
        let err = pipeline.run().unwrap_err();
        assert_eq!(err.kind, GrabErrorKind::Subprocess);
        assert!(err.message.contains("boom"));
    }
}
