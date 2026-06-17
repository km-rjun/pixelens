//! Unix domain socket server.
//!
//! Listens on `$XDG_RUNTIME_DIR/pixelens.sock` (falling back to
//! `/tmp/pixelens-$UID.sock` when `$XDG_RUNTIME_DIR` is unset). Each
//! connection is a single request/response exchange handled by
//! [`dispatch`]. Connections are short-lived — the daemon does not
//! hold any per-connection state.

use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::{UnixListener, UnixStream};

use pixelens_ipc::{read_frame, write_response, FrameError, IpcRequest};

use crate::dispatch::Dispatcher;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("could not determine socket path: {0}")]
    NoSocketPath(String),

    #[error("bind failed: {0}")]
    Bind(#[from] std::io::Error),

    #[error("frame error: {0}")]
    Frame(#[from] FrameError),
}

/// Resolve the socket path. Honours `$XDG_RUNTIME_DIR` per the PRD, and
/// falls back to `/tmp/pixelens-${UID}.sock` (using the current uid)
/// for systems that don't set it (notably some `sudo` invocations).
pub fn socket_path() -> Result<PathBuf, ServerError> {
    if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir).join("pixelens.sock"));
        }
    }

    let uid = nix_current_uid().ok_or_else(|| {
        ServerError::NoSocketPath(
            "neither XDG_RUNTIME_DIR nor a usable uid is available".to_string(),
        )
    })?;
    Ok(PathBuf::from(format!("/tmp/pixelens-{uid}.sock")))
}

#[cfg(unix)]
fn nix_current_uid() -> Option<u32> {
    // We avoid pulling in the `nix` crate for one libc call.
    extern "C" {
        fn getuid() -> u32;
    }
    // SAFETY: getuid is async-signal-safe and has no preconditions.
    Some(unsafe { getuid() })
}

#[cfg(not(unix))]
fn nix_current_uid() -> Option<u32> {
    None
}

/// Start the IPC server. Returns once the listener is bound; the
/// caller spawns the accept loop on a tokio task.
pub async fn bind() -> Result<(UnixListener, PathBuf), ServerError> {
    let path = socket_path()?;

    // Remove any stale socket file from a previous (crashed) run.
    // `remove_file` is best-effort: ENOENT is fine.
    match std::fs::remove_file(&path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(ServerError::Bind(e)),
    }

    let listener = UnixListener::bind(&path)?;
    tracing::info!(socket = %path.display(), "ipc listener bound");
    Ok((listener, path))
}

/// Accept and serve connections forever.
pub async fn serve(listener: UnixListener, dispatcher: Arc<Dispatcher>) {
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let dispatcher = Arc::clone(&dispatcher);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, dispatcher).await {
                        tracing::warn!(error = %e, "connection handler exited with error");
                    }
                });
            }
            Err(e) => {
                tracing::error!(error = %e, "accept failed");
            }
        }
    }
}

async fn handle_connection(
    mut stream: UnixStream,
    dispatcher: Arc<Dispatcher>,
) -> Result<(), ServerError> {
    let request: IpcRequest = read_frame(&mut stream).await?;
    tracing::debug!(
        command = ?request.command,
        request_id = %request.request_id,
        "received request"
    );

    let response = dispatcher.dispatch(request).await;
    write_response(&response, &mut stream).await?;
    Ok(())
}
