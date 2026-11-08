//! System clipboard write (M7).
//!
//! Dependency-light approach: shell out to the platform clipboard
//! tool rather than pulling in a native clipboard crate. The backend
//! is chosen by the detected display server:
//!
//! - **Wayland**: `wl-copy` (from `wl-clipboard`), with `copyq`
//!   as a fallback.
//! - **X11**: `xclip -selection clipboard` or `xsel -b`.
//! - **Windows**: Uses `arboard` via the `pixelens-notify` crate.
//!
//! If no usable clipboard tool is found, [`copy_text`] returns
//! [`ClipboardError::NoBackend`]. Callers are expected to log a
//! warning and continue — a missing clipboard tool must never fail a
//! successful capture/OCR (PRD core loop is best-effort).

use std::io::Write;
use std::process::{Command, Stdio};

#[cfg(unix)]
use pixelens_capture::DisplayServer;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClipboardError {
    /// No clipboard backend (wl-copy/xclip/xsel/copyq) found on $PATH.
    #[error("clipboard unavailable: install wl-clipboard (Wayland) or xclip/xsel (X11)")]
    NoBackend,

    /// The chosen backend exited with a non-zero status.
    #[error("clipboard backend failed: {0}")]
    Backend(String),

    /// Failed to spawn the backend subprocess.
    #[error("failed to spawn clipboard backend: {0}")]
    Spawn(#[from] std::io::Error),
}

#[cfg(unix)]
/// Copy `text` to the system clipboard, selecting a backend based on
/// the active [`DisplayServer`].
///
/// Never succeeds-fails the grab: callers should treat a returned
/// `Err` as "log + continue".
pub fn copy_text(text: &str, display: DisplayServer) -> Result<(), ClipboardError> {
    match display {
        DisplayServer::Wayland => copy_wayland(text),
        DisplayServer::X11 => copy_x11(text),
    }
}

#[cfg(windows)]
/// Copy `text` to the system clipboard on Windows using the native API.
///
/// Never succeeds-fails the grab: callers should treat a returned
/// `Err` as "log + continue".
pub fn copy_text(text: &str) -> Result<(), ClipboardError> {
    use pixelens_notify::{windows_clipboard_copy, NotifyError};
    windows_clipboard_copy(text).map_err(|e| ClipboardError::Backend(e.to_string()))
}

#[cfg(unix)]
/// Try a list of candidate backends in order; return on the first
/// that runs successfully.
fn copy_with_candidates(candidates: &[(&str, &[&str])], text: &str) -> Result<(), ClipboardError> {
    let mut last_err = ClipboardError::NoBackend;
    for (bin, args) in candidates {
        // Check the binary exists before trying (so we skip cleanly to
        // the next candidate instead of producing a spurious spawn error).
        if Command::new(bin)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            match run_backend(bin, args, text) {
                Ok(()) => return Ok(()),
                Err(e) => last_err = e,
            }
        }
    }
    Err(last_err)
}

fn run_backend(bin: &str, args: &[&str], text: &str) -> Result<(), ClipboardError> {
    let mut child = Command::new(bin)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    // Feed text to the backend's stdin (all supported tools read stdin).
    {
        let mut stdin = child.stdin.take().expect("stdin was piped");
        stdin
            .write_all(text.as_bytes())
            .map_err(ClipboardError::Spawn)?;
    }

    let output = child.wait_with_output().map_err(ClipboardError::Spawn)?;
    if output.status.success() {
        Ok(())
    } else {
        let msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(ClipboardError::Backend(format!(
            "{bin} exited with status {}: {msg}",
            output.status
        )))
    }
}

#[cfg(unix)]
fn copy_wayland(text: &str) -> Result<(), ClipboardError> {
    copy_with_candidates(
        &[
            // wl-clipboard
            ("wl-copy", &[] as &[&str]),
            // copyq (Wayland-capable clipboard manager)
            ("copyq", &["copy", "-"]),
        ],
        text,
    )
}

#[cfg(unix)]
fn copy_x11(text: &str) -> Result<(), ClipboardError> {
    copy_with_candidates(
        &[
            // xclip: read stdin into the CLIPBOARD selection
            ("xclip", &["-selection", "clipboard", "-in"]),
            // xsel: read stdin into the CLIPBOARD selection
            ("xsel", &["--clipboard", "--input"]),
            // copyq as a last resort
            ("copyq", &["copy", "-"]),
        ],
        text,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_text_returns_no_backend_without_clipboard_tools() {
        // On a headless CI box none of wl-copy/xclip/xsel/copyq should
        // be present, so copy_text must degrade to NoBackend rather than
        // panic. We assert the error is the graceful NoBackend variant.
        #[cfg(unix)]
        {
            let err = copy_text("hello", DisplayServer::Wayland);
            assert!(
                matches!(err, Err(ClipboardError::NoBackend)),
                "got: {err:?}"
            );

            let err = copy_text("hello", DisplayServer::X11);
            assert!(
                matches!(err, Err(ClipboardError::NoBackend)),
                "got: {err:?}"
            );
        }
        #[cfg(windows)]
        {
            // On Windows the native clipboard via arboard is always available,
            // so copy_text should succeed.
            let result = copy_text("hello");
            assert!(result.is_ok(), "expected Ok on Windows, got: {result:?}");
        }
    }
}
