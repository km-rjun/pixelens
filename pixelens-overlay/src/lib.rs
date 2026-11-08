//! Region selection overlay.
//!
//! Splits display-server-coupled UI from notification plumbing
//! (PRD §"Repository Structure" — `pixelens-ui` was split into
//! `pixelens-overlay` and `pixelens-notify`).
//!
//! M1: trait only. The layer-shell implementation (M3) and the XCB
//! implementation (M4) live in this crate.

use pixelens_core::{PixelensError, Rect};

/// Backend-agnostic overlay surface used by the capture engine.
///
/// The overlay must appear in <100 ms of the hotkey (PRD §"Performance")
/// and must vanish cleanly on cancel with no on-screen artefacts.
pub trait SelectionOverlay: Send + Sync {
    /// Show the overlay and block until the user releases a selection
    /// or cancels. Returns `Ok(None)` on cancel.
    fn run(&self, session_id: &str) -> Result<Option<Rect>, PixelensError>;
}

/// Placeholder Wayland overlay (layer-shell). Real impl in M3.
pub struct WaylandOverlay;

/// Placeholder X11 overlay (XCB InputOnly). Real impl in M4.
pub struct X11Overlay;
