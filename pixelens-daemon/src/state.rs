//! Daemon-wide state shared between the IPC server and the dispatcher.
//!
//! In v1 the state is small: a snapshot of the detected display server,
//! an optional capture pipeline, and an optional warm OCR engine. The
//! pipeline may be absent if `slurp` / `grim` aren't installed; in that
//! case `pixelens grab` returns a clear `MissingTool` error and the
//! daemon keeps running for the other commands. Similarly the OCR engine
//! is `None` when `tesseract` is unavailable — grabs still succeed, they
//! just return no extracted text (M5).

use std::sync::{Arc, Mutex};

use pixelens_capture::{CaptureBackend, DisplayServer, GrabPipeline};
use pixelens_config::Config;
use pixelens_menu::MenuBackend;
use pixelens_ocr::TesseractOcrEngine;

/// An optional menu backend override. `None` (production default) lets the
/// dispatcher auto-detect one via [`pixelens_menu::detect_backend`]; tests and
/// embedders inject a concrete backend here to avoid touching global state
/// (stdin / PATH / DBus).
pub type MenuOverride = Arc<dyn MenuBackend + Send + Sync + 'static>;

/// One-shot GUI/UM4 state mutated via IPC (`SetPreview`, `Redetect`) and
/// consumed by the next grab. Behind a mutex so the IPC handler and the
/// dispatcher can both touch it without refactoring the `Arc<DaemonState>`
/// into an inner `Arc<Mutex<...>>`.
#[derive(Debug, Default)]
pub struct OneShot {
    /// Override `capture.show_preview` for exactly the next grab. `None`
    /// means "use config". Cleared to `None` after the grab reads it.
    pub preview: Option<bool>,
    /// `true` once `Redetect` is requested; the daemon loop (or next
    /// grab) re-queries outputs and resets this to `false`.
    pub redetect: bool,
}

pub struct DaemonState {
    pub display: DisplayServer,
    /// `None` when the pipeline failed to construct (e.g. slurp/grim
    /// missing). All other commands are unaffected; only `grab`
    /// surfaces this.
    pub pipeline: Option<GrabPipeline>,
    /// Warm OCR engine. `None` when `tesseract` is unavailable; grabs
    /// still work, they simply return empty `text` (M5 degrade path).
    pub ocr: Option<TesseractOcrEngine>,
    /// UM5: optional portal-native capture backend. When `Some`, grabs
    /// prefer this fast-path over the slurp/grim `pipeline`. The backend
    /// itself transparently falls back to slurp/grim if the portal
    /// session is unavailable, so `pipeline` remains a valid fallback.
    /// `None` in default builds (no `portal` feature) or when selection
    /// yields no backend.
    pub portal_backend: Option<Arc<CaptureBackend>>,
    /// Parsed on-disk config (UM4: `gui` + `capture.show_preview` base).
    pub config: Config,
    /// UM4 one-shot overrides, shared with the IPC handler.
    pub one_shot: Arc<Mutex<OneShot>>,
    /// u8: optional injected menu backend. `None` => dispatcher auto-detects.
    pub menu_override: Option<MenuOverride>,
}

impl DaemonState {
    pub fn new(
        display: DisplayServer,
        pipeline: Option<GrabPipeline>,
        ocr: Option<TesseractOcrEngine>,
        config: Config,
        portal_backend: Option<Arc<CaptureBackend>>,
        menu_override: Option<MenuOverride>,
    ) -> Self {
        Self {
            display,
            pipeline,
            ocr,
            portal_backend,
            config,
            one_shot: Arc::new(Mutex::new(OneShot::default())),
            menu_override,
        }
    }

    /// Effective preview flag for the *next* grab: the one-shot override
    /// wins over `capture.show_preview` from config. Does NOT consume the
    /// override — call [`DaemonState::take_preview_override`] after the
    /// grab to clear it.
    pub fn preview_for_next_grab(&self) -> bool {
        let guard = self.one_shot.lock().expect("one_shot mutex poisoned");
        guard.preview.unwrap_or(self.config.capture.show_preview)
    }

    /// Consume the one-shot preview override (set back to `None`) so the
    /// next grab reverts to config. Returns the value that was in effect.
    pub fn take_preview_override(&self) -> Option<bool> {
        let mut guard = self.one_shot.lock().expect("one_shot mutex poisoned");
        std::mem::take(&mut guard.preview)
    }

    /// Set the one-shot preview override.
    pub fn set_preview_override(&self, preview: bool) {
        let mut guard = self.one_shot.lock().expect("one_shot mutex poisoned");
        guard.preview = Some(preview);
    }

    /// Mark that a re-detect of outputs was requested.
    pub fn request_redetect(&self) {
        let mut guard = self.one_shot.lock().expect("one_shot mutex poisoned");
        guard.redetect = true;
    }

    /// Consume the re-detect flag.
    pub fn take_redetect(&self) -> bool {
        let mut guard = self.one_shot.lock().expect("one_shot mutex poisoned");
        std::mem::replace(&mut guard.redetect, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pixelens_config::Config;

    fn state_with(show_preview: bool) -> DaemonState {
        let mut config = Config::default();
        config.capture.show_preview = show_preview;
        DaemonState::new(DisplayServer::Wayland, None, None, config, None, None)
    }

    #[test]
    fn no_override_uses_config() {
        let s = state_with(false);
        assert!(!s.preview_for_next_grab());
        // idempotent: reading it does not consume
        assert!(!s.preview_for_next_grab());
    }

    #[test]
    fn override_wins_then_reverts() {
        let s = state_with(false);
        s.set_preview_override(true);
        assert!(s.preview_for_next_grab());
        // consume the one-shot
        assert_eq!(s.take_preview_override(), Some(true));
        // next grab falls back to config (false)
        assert!(!s.preview_for_next_grab());
    }

    #[test]
    fn override_suppresses_config_true() {
        let s = state_with(true);
        s.set_preview_override(false);
        assert!(!s.preview_for_next_grab());
        assert_eq!(s.take_preview_override(), Some(false));
        // reverts to config true
        assert!(s.preview_for_next_grab());
    }

    #[test]
    fn redetect_flag_round_trips() {
        let s = state_with(false);
        assert!(!s.take_redetect());
        s.request_redetect();
        assert!(s.take_redetect());
        assert!(!s.take_redetect());
    }
}
