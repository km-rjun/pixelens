//! Length-prefixed JSON framing.
//!
//! The wire format is `[u32 BE length][N bytes JSON]`. A sanity cap
//! prevents a malicious or buggy peer from making us allocate arbitrarily
//! large buffers.
//!
//! Transport-agnostic: the framing works over any `AsyncRead + AsyncWrite`
//! stream (Unix socket, TCP, in-memory test pipe). M6 will plug it into
//! `tokio::net::UnixStream` for the real server and client.

use std::io;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub use super::protocol::{IpcError, IpcRequest, IpcResponse};

/// Named pipe path used by the Windows daemon/client. Resolved by
/// [`windows_pipe_path`] so the string stays a single source of truth.
pub const WINDOWS_PIPE_NAME: &str = "pixelens";

/// Windows named-pipe full path: `\\.\pipe\pixelens`.
pub fn windows_pipe_path() -> String {
    format!("\\\\.\\pipe\\{WINDOWS_PIPE_NAME}")
}

/// A transport-agnostic duplex stream over which the codec frames.
#[cfg(unix)]
pub type IpcStream = tokio::net::UnixStream;

/// A transport-agnostic duplex stream over which the codec frames.
#[cfg(windows)]
pub type IpcStream = tokio::net::windows::named_pipe::NamedPipeClient;

/// Connect to the running daemon. Unix uses a `UnixStream`; Windows uses
/// the `\\.\pipe\pixelens` named pipe. The codec on top is identical.
pub async fn connect() -> Result<IpcStream, FrameError> {
    #[cfg(unix)]
    {
        let path = super::socket_path();
        Ok(tokio::net::UnixStream::connect(path).await?)
    }
    #[cfg(windows)]
    {
        use tokio::net::windows::named_pipe::ClientOptions;
        let client = ClientOptions::new()
            .open(windows_pipe_path())
            .map_err(FrameError::Io)?;
        Ok(client)
    }
}

/// Bind a server-side listener. Unix returns a `UnixListener`; Windows a
/// `NamedPipeServer`. Listener types differ, so this stays typed per-OS.
#[cfg(unix)]
pub async fn bind() -> Result<tokio::net::UnixListener, FrameError> {
    let path = super::socket_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    Ok(tokio::net::UnixListener::bind(&path)?)
}

/// Bind a server-side named-pipe listener on Windows.
#[cfg(windows)]
pub async fn bind() -> Result<tokio::net::windows::named_pipe::NamedPipeServer, FrameError> {
    use tokio::net::windows::named_pipe::ServerOptions;
    let server = ServerOptions::new()
        .create(windows_pipe_path())
        .map_err(FrameError::Io)?;
    Ok(server)
}

/// 16 MiB. Generous enough for any reasonable config or status payload,
/// small enough that a misbehaving peer can't OOM us.
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum FrameError {
    #[error(transparent)]
    Ipc(#[from] IpcError),

    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

/// Read a single length-prefixed JSON request from `stream`.
pub async fn read_frame<S>(stream: &mut S) -> Result<IpcRequest, FrameError>
where
    S: AsyncReadExt + Unpin,
{
    let body = read_frame_body(stream).await?;
    let req: IpcRequest = serde_json::from_slice(&body).map_err(IpcError::from)?;
    Ok(req)
}

/// Read a single length-prefixed JSON response from `stream`.
pub async fn read_response<S>(stream: &mut S) -> Result<IpcResponse, FrameError>
where
    S: AsyncReadExt + Unpin,
{
    let body = read_frame_body(stream).await?;
    let resp: IpcResponse = serde_json::from_slice(&body).map_err(IpcError::from)?;
    Ok(resp)
}

/// Write a single length-prefixed JSON request to `stream`.
pub async fn write_frame<S>(request: &IpcRequest, stream: &mut S) -> Result<(), FrameError>
where
    S: AsyncWriteExt + Unpin,
{
    let body = serde_json::to_vec(request).map_err(IpcError::from)?;
    write_frame_body(&body, stream).await
}

/// Write a single length-prefixed JSON response to `stream`.
pub async fn write_response<S>(response: &IpcResponse, stream: &mut S) -> Result<(), FrameError>
where
    S: AsyncWriteExt + Unpin,
{
    let body = serde_json::to_vec(response).map_err(IpcError::from)?;
    write_frame_body(&body, stream).await
}

async fn read_frame_body<S>(stream: &mut S) -> Result<Vec<u8>, FrameError>
where
    S: AsyncReadExt + Unpin,
{
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > MAX_FRAME_SIZE {
        return Err(FrameError::Ipc(IpcError::FrameTooLarge {
            size: len,
            max: MAX_FRAME_SIZE,
        }));
    }

    let mut body = vec![0u8; len];
    stream.read_exact(&mut body).await?;
    Ok(body)
}

async fn write_frame_body<S>(body: &[u8], stream: &mut S) -> Result<(), FrameError>
where
    S: AsyncWriteExt + Unpin,
{
    if body.len() > MAX_FRAME_SIZE {
        return Err(FrameError::Ipc(IpcError::FrameTooLarge {
            size: body.len(),
            max: MAX_FRAME_SIZE,
        }));
    }

    let len = (body.len() as u32).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(body).await?;
    stream.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{Command, ResponseStatus};

    use super::*;

    #[test]
    fn windows_pipe_path_is_canonical() {
        assert_eq!(windows_pipe_path(), "\\\\.\\pipe\\pixelens");
    }

    // Round-trip the codec over an in-memory duplex pair. Transport-agnostic:
    // works unchanged on any `AsyncRead + AsyncWrite` stream, including a
    // Windows named pipe.
    #[tokio::test]
    async fn codec_round_trips_request_and_response() {
        let (mut a, mut b) = tokio::io::duplex(4096);

        let req = IpcRequest {
            request_id: "req-1".to_string(),
            command: Command::Grab,
            payload: serde_json::json!({ "region": "full" }),
        };
        write_frame(&req, &mut a).await.unwrap();

        let received = read_frame(&mut b).await.unwrap();
        assert_eq!(received.request_id, "req-1");
        assert_eq!(received.command, Command::Grab);
        assert_eq!(received.payload["region"], "full");

        let resp = IpcResponse {
            request_id: "req-1".to_string(),
            status: ResponseStatus::Ok,
            payload: serde_json::json!({ "text": "hello" }),
        };
        write_response(&resp, &mut b).await.unwrap();
        let received_resp = read_response(&mut a).await.unwrap();
        assert_eq!(received_resp.status, ResponseStatus::Ok);
        assert_eq!(received_resp.payload["text"], "hello");
    }

    #[tokio::test]
    async fn oversized_frame_is_rejected() {
        let (mut a, mut b) = tokio::io::duplex(4096);
        // Claim a body far larger than MAX_FRAME_SIZE.
        let len = (MAX_FRAME_SIZE as u32 + 1).to_be_bytes();
        a.write_all(&len).await.unwrap();
        a.flush().await.unwrap();
        let err = read_frame_body(&mut b).await.unwrap_err();
        assert!(matches!(
            err,
            FrameError::Ipc(IpcError::FrameTooLarge { .. })
        ));
    }
}
