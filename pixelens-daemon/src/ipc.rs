//! IPC server abstraction over Unix sockets and Windows named pipes.
//!
//! This module wraps the transport-agnostic types from `pixelens_ipc`
//! and provides the server-side accept loop.

use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

use pixelens_ipc::{read_frame, write_response, FrameError, IpcRequest, windows_pipe_path};
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
    extern "C" { fn getuid() -> u32; }
    Some(unsafe { getuid() })
}

#[cfg(not(unix))]
fn nix_current_uid() -> Option<u32> {
    None
}

/// Cross-platform listener type that wraps the platform-specific listener.
/// Uses the same return types as `pixelens_ipc::bind()`.
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
            if let IpcListener::Unix(l) = self {
                return PathBuf::from(l.local_addr().map(|a| a.as_pathname().display().to_string()).unwrap_or("(unknown)".into()));
            }
        }
        #[cfg(windows)]
        {
            if let IpcListener::Windows(_) = self {
                return PathBuf::from(windows_pipe_path());
            }
        }
        PathBuf::from("(unknown)")
    }
}

/// Start the IPC server. Returns a bound listener.
/// Cross-platform: delegates to `pixelens_ipc::bind()`.
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

/// Accept and serve connections forever.
pub async fn serve(listener: IpcListener, dispatcher: Arc<Dispatcher>) {
    #[cfg(unix)]
    {
        if let IpcListener::Unix(listener) = listener {
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
    }

    #[cfg(windows)]
    {
        if let IpcListener::Windows(server) = listener {
            loop {
                match server.connect().await {
                    Ok(client) => {
                        let dispatcher = Arc::clone(&dispatcher);
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(client, dispatcher).await {
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