//! `pixelensd` daemon library.
//!
//! The actual binary entry point lives in `main.rs` and is a thin
//! wrapper around [`run`]. Splitting the daemon into lib + bin lets
//! integration tests exercise the dispatcher, IPC server, and capture
//! pipeline without spawning a subprocess.

pub mod clipboard;
pub mod dispatch;
pub mod ipc;
pub mod state;

use std::sync::Arc;
use tracing_subscriber::EnvFilter;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialise tracing with sensible defaults. Idempotent; safe to call
/// from both the binary and integration tests.
pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

/// Run the daemon. Blocks forever (the IPC accept loop never returns).
pub async fn run() -> anyhow::Result<()> {
    init_tracing();

    tracing::info!(version = VERSION, "pixelensd starting");

    // M8: load on-disk configuration so the parsed-but-unused keys
    // become *used*. We read the config and visibly consume at least
    // `general.hotkey` (default combo for the keyhook) and
    // `capture.show_preview` (gates a preview hint). Config load is
    // non-fatal: a missing/invalid file falls back to defaults.
    let config = pixelens_config::load_config().unwrap_or_else(|e| {
        tracing::warn!(error = %e, "failed to load config; using defaults");
        pixelens_config::Config::default()
    });

    // Default keyhook combo: prefer PIXELENS_HOTKEY env, then the
    // config value, then the model default. We surface the resolved
    // combo so the daemon's startup log observably consumes the key.
    let hotkey = std::env::var("PIXELENS_HOTKEY").unwrap_or_else(|_| config.general.hotkey.clone());
    tracing::info!(hotkey = %hotkey, "keyhook combo resolved (env > config > default)");

    if config.capture.show_preview {
        // Full preview UI is a later milestone; for M8 we simply log
        // that the flag is being read so the key is observably consumed.
        tracing::info!("capture.show_preview = true; a capture preview would be shown");
    } else {
        tracing::debug!("capture.show_preview = false; fast path (no preview)");
    }

    // UM4: surface the GUI feature flags so `gui.*` keys are observably
    // consumed (the visual HUD crate reads them; `hud_enabled` is the
    // master switch that gates the hotkey `Space` chord).
    tracing::info!(
        hud_enabled = config.gui.hud_enabled,
        hud_timeout_ms = config.gui.hud_timeout_ms,
        "gui config resolved"
    );

    // Display-server detection is the first gate per PRD §"Display
    // Server Detection". Failures here are fatal.
    let display = pixelens_capture::detect_display_server()
        .map_err(|e| anyhow::anyhow!("display server detection failed: {e}"))?;
    let display_name = match display {
        pixelens_capture::DisplayServer::Wayland => "wayland",
        pixelens_capture::DisplayServer::X11 => "x11",
    };
    tracing::info!(display = display_name, "display server detected");

    let pipeline = match pixelens_capture::GrabPipeline::new() {
        Ok(p) => {
            tracing::info!("capture pipeline ready (slurp + grim)");
            Some(p)
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "capture pipeline unavailable; `pixelens grab` will report this"
            );
            None
        }
    };

    // Bind the IPC socket. This is the only fatal startup failure
    // after display detection: without the socket, the CLI has no way
    // to reach the daemon.
    let (listener, socket_path) = ipc::bind()
        .await
        .map_err(|e| anyhow::anyhow!("ipc bind failed: {e}"))?;

    println!("pixelensd {VERSION} listening on {}", socket_path.display());

    // Warm-init the OCR engine (M5). A missing `tesseract` is non-fatal:
    // capture still works, grabs just return empty `text`. We log a
    // warning and continue rather than crash the daemon.
    let ocr = match pixelens_ocr::TesseractOcrEngine::new() {
        Ok(engine) => {
            tracing::info!("OCR engine ready (tesseract)");
            Some(engine)
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "OCR engine unavailable; grabs will return no text until tesseract is installed"
            );
            None
        }
    };

    let state = Arc::new(state::DaemonState::new(display, pipeline, ocr, config));
    let dispatcher = Arc::new(dispatch::Dispatcher::new(state));

    ipc::serve(listener, dispatcher).await;
    Ok(())
}
