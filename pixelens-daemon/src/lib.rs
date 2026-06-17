//! `pixelensd` daemon library.
//!
//! The actual binary entry point lives in `main.rs` and is a thin
//! wrapper around [`run`]. Splitting the daemon into lib + bin lets
//! integration tests exercise the dispatcher, IPC server, and capture
//! pipeline without spawning a subprocess.

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

    let state = Arc::new(state::DaemonState::new(display, pipeline));
    let dispatcher = Arc::new(dispatch::Dispatcher::new(state));

    ipc::serve(listener, dispatcher).await;
    Ok(())
}
