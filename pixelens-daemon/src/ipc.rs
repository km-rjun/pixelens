//! IPC server abstraction over Unix sockets and Windows named pipes.
//!
//! On Unix: listens on `$XDG_RUNTIME_DIR/pixelens.sock` (falling back to
//! `/tmp/pixelens-$UID.sock`). On Windows: listens on the named pipe
//! `\\.\pipe\pixelens`. Each connection is a single request/response exchange
//! handled by [`dispatch`]. Connections are short-lived — the daemon does not
//! hold any per-connection state.

use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

use crate::dispatch::Dispatcher;
#[cfg(windows)]
use pixelens_ipc::{read_frame, windows_pipe_path, write_response, FrameError, IpcRequest};
#[cfg(unix)]
use pixelens_ipc::{read_frame, write_response, FrameError, IpcRequest};

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
/// On Windows this returns a placeholder since the endpoint is a named pipe.
pub fn socket_path() -> Result<PathBuf, ServerError> {
    #[cfg(unix)]
    {
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

    #[cfg(not(unix))]
    {
        // On Windows we use a named pipe; this path is not used for binding.
        Ok(PathBuf::from("(named pipe)"))
    }
}

#[cfg(unix)]
fn nix_current_uid() -> Option<u32> {
    extern "C" {
        fn getuid() -> u32;
    }
    Some(unsafe { getuid() })
}

#[cfg(not(unix))]
fn nix_current_uid() -> Option<u32> {
    None
}

/// Cross-platform listener type that wraps the platform-specific listener.
pub enum IpcListener {
    #[cfg(unix)]
    Unix(tokio::net::UnixListener),
    #[cfg(windows)]
    Windows(tokio::net::windows::named_pipe::NamedPipeServer),
}

impl IpcListener {
    /// Return a displayable path for logging.
    pub fn socket_path(&self) -> PathBuf {
        #[cfg(unix)]
        {
            let IpcListener::Unix(l) = self;
            l.local_addr()
                .ok()
                .and_then(|a| a.as_pathname().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("(unknown)"))
        }
        #[cfg(windows)]
        {
            let IpcListener::Windows(_) = self;
            PathBuf::from(windows_pipe_path())
        }
        #[cfg(not(any(unix, windows)))]
        PathBuf::from("(unknown)")
    }
}

/// Start the IPC server. Returns a bound listener.
/// Cross-platform: delegates to platform-specific bind.
pub async fn bind() -> Result<IpcListener, ServerError> {
    #[cfg(unix)]
    {
        let path = socket_path()?;

        // Remove any stale socket file from a previous (crashed) run.
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(ServerError::Bind(e)),
        }

        let listener = tokio::net::UnixListener::bind(&path)?;
        tracing::info!(socket = %path.display(), "ipc listener bound");
        Ok(IpcListener::Unix(listener))
    }

    #[cfg(windows)]
    {
        use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
        let pipe_path = windows_pipe_path();
        let server: NamedPipeServer = ServerOptions::new()
            .create(&pipe_path)
            .map_err(FrameError::Io)?;
        tracing::info!(pipe = %pipe_path, "ipc named pipe listener bound");
        Ok(IpcListener::Windows(server))
    }
}

/// Accept and serve connections until shutdown signal received.
pub async fn serve(
    listener: IpcListener,
    dispatcher: Arc<Dispatcher>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) {
    #[cfg(unix)]
    {
        let IpcListener::Unix(listener) = listener;
        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
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
                _ = shutdown_rx.recv() => {
                    tracing::info!("shutdown signal received, stopping IPC server");
                    break;
                }
            }
        }
    }

    #[cfg(windows)]
    {
        let IpcListener::Windows(mut server) = listener;
        loop {
            tokio::select! {
                connect_result = server.connect() => {
                    if let Err(e) = connect_result {
                        tracing::error!(error = %e, "client connect failed");
                        continue;
                    }

                    // Spawn a task to handle this connection using the server as the stream.
                    let dispatcher = Arc::clone(&dispatcher);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(server, dispatcher).await {
                            tracing::warn!(error = %e, "connection handler exited with error");
                        }
                    });

                    // Create a new server for the next connection
                    use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
                    let pipe_path = windows_pipe_path();
                    server = match ServerOptions::new().create(&pipe_path) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!(error = %e, "failed to recreate named pipe server");
                            break;
                        }
                    };
                }
                _ = shutdown_rx.recv() => {
                    tracing::info!("shutdown signal received, stopping IPC server");
                    break;
                }
            }
        }
    }
}

async fn handle_connection<S>(mut stream: S, dispatcher: Arc<Dispatcher>) -> Result<(), ServerError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
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
