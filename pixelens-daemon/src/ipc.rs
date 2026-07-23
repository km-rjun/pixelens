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

use pixelens_ipc::{read_frame, write_response, FrameError, IpcRequest, IpcStream};

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
/// Cross-platform: returns a platform-specific listener wrapped in IpcListener.
pub async fn bind() -> Result<IpcListener, ServerError> {
    #[cfg(unix)]
    {
        use tokio::net::UnixListener;
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
        Ok(IpcListener::Unix(listener))
    }

    #[cfg(windows)]
    {
        use tokio::net::windows::named_pipe::ServerOptions;
        use pixelens_ipc::windows_pipe_path;
        let pipe_path = windows_pipe_path();
        let server = ServerOptions::new()
            .create(pipe_path)
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
                match server.accept().await {
                    Ok(stream) => {
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
}

async fn handle_connection(
    mut stream: IpcStream,
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

// Cross-platform listener type.
#[cfg(unix)]
pub enum IpcListener {
    Unix(tokio::net::UnixListener),
    Windows(tokio::net::windows::named_pipe::NamedPipeServer),
}

#[cfg(windows)]
pub enum IpcListener {
    Unix(tokio::net::UnixListener),
    Windows(tokio::net::windows::named_pipe::NamedPipeServer),
}

// Cross-platform stream type.
#[cfg(unix)]
pub enum IpcStream {
    Unix(tokio::net::UnixStream),
    Windows(tokio::net::windows::named_pipe::NamedPipeClient),
}

#[cfg(windows)]
pub enum IpcStream {
    Unix(tokio::net::UnixStream),
    Windows(tokio::net::windows::named_pipe::NamedPipeClient),
}

// Implement AsyncRead/AsyncWrite for IpcStream to delegate to the inner stream.
impl tokio::io::AsyncRead for IpcStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self {
            IpcStream::Unix(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            IpcStream::Windows(s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl tokio::io::AsyncWrite for IpcStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match &mut *self {
            IpcStream::Unix(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            IpcStream::Windows(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self {
            IpcStream::Unix(s) => std::pin::Pin::new(s).poll_flush(cx),
            IpcStream::Windows(s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut *self {
            IpcStream::Unix(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            IpcStream::Windows(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}

impl Unpin for IpcStream {}