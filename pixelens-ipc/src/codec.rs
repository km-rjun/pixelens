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
