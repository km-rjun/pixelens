//! `pixelensd` — the Pixelens background daemon binary entry point.
//!
//! All non-trivial logic lives in the [`pixelens_daemon`] library so
//! integration tests can exercise it without spawning a subprocess.
//! This file is a thin wrapper that calls [`pixelens_daemon::run`].

use tokio::signal;
use tokio::sync::broadcast;

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // Create shutdown signal channel
    let (shutdown_tx, _) = broadcast::channel(1);
    let shutdown_tx_clone = shutdown_tx.clone();

    // Handle shutdown signals
    let shutdown_handle = rt.spawn(async move {
        #[cfg(unix)]
        {
            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to register SIGTERM handler");
            let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
                .expect("failed to register SIGINT handler");

            tokio::select! {
                _ = signal::ctrl_c() => {
                    tracing::info!("received Ctrl+C, shutting down");
                }
                _ = sigterm.recv() => {
                    tracing::info!("received SIGTERM, shutting down");
                }
                _ = sigint.recv() => {
                    tracing::info!("received SIGINT, shutting down");
                }
            };
        }
        #[cfg(not(unix))]
        {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    tracing::info!("received Ctrl+C, shutting down");
                }
            };
        }
        let _ = shutdown_tx_clone.send(());
    });

    let result = rt.block_on(pixelens_daemon::run(shutdown_tx.subscribe()));

    // Wait for shutdown handler to complete
    let _ = rt.block_on(shutdown_handle);

    result
}
