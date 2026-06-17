//! `pixelensd` — the Pixelens background daemon.
//!
//! M1: entry point that wires tracing and prints a placeholder banner.
//! M2 will add display-server detection at startup. M5 will add the
//! Tesseract validation step. M6 will bind the IPC socket. M9 will
//! add the tray.

use pixelens_capture::detect_display_server;
use pixelens_core::PixelensError;
use tracing_subscriber::EnvFilter;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    init_tracing();

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("pixelensd {}", VERSION);
        return;
    }

    let daemonized = args.iter().any(|a| a == "--daemon");

    tracing::info!(version = VERSION, daemonized, "pixelensd starting");

    // M2 will move display detection up to the very first action and
    // gate every subsystem on its result. For M1 we still call it so
    // the wiring is exercised end-to-end.
    match detect_display_server() {
        Ok(server) => tracing::info!(?server, "display server detected"),
        Err(PixelensError::NoDisplayServer) => {
            tracing::error!("no display server detected; refusing to start");
            std::process::exit(1);
        }
        Err(e) => {
            tracing::error!(error = %e, "display server detection failed");
            std::process::exit(1);
        }
    }

    println!("pixelensd {} (scaffold)", VERSION);
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}
