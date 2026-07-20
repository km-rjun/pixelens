//! Length-prefixed JSON IPC over a Unix domain socket.
//!
//! Wire format (per PRD §"IPC"):
//!
//! ```text
//! [4 bytes: u32 big-endian payload length][N bytes: UTF-8 JSON]
//! ```
//!
//! Every request carries a `request_id` (UUIDv4) that the response echoes.
//! This lets the daemon match a `cancel` command to the in-flight session
//! it belongs to. M6 fleshes out the socket server and client; M1 only
//! declares the framing primitives and message types so the rest of the
//! workspace compiles against them.

pub mod codec;
pub mod protocol;

/// Resolve the daemon socket path. On Unix this mirrors the CLI/keyhook
/// resolution (XDG_RUNTIME_DIR with a uid fallback). On Windows the IPC
/// transport is a named pipe, so this is unused — `codec::windows_pipe_path`
/// is the Windows endpoint instead.
pub fn socket_path() -> std::path::PathBuf {
    #[cfg(unix)]
    {
        if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            if !dir.is_empty() {
                return std::path::PathBuf::from(dir).join("pixelens.sock");
            }
        }
        extern "C" {
            fn getuid() -> u32;
        }
        // SAFETY: getuid is async-signal-safe and has no preconditions.
        let uid = unsafe { getuid() };
        std::path::PathBuf::from(format!("/tmp/pixelens-{uid}.sock"))
    }
    #[cfg(not(unix))]
    {
        std::path::PathBuf::from("(no unix socket)")
    }
}

pub use codec::{
    bind, connect, read_frame, read_response, windows_pipe_path, write_frame, write_response,
    FrameError, IpcStream, MAX_FRAME_SIZE,
};
pub use protocol::{
    AiPayload, AiResponsePayload, Command, GrabRegion, GrabResponsePayload, IpcError, IpcRequest,
    IpcResponse, RequestId, ResponseStatus, SetPreviewPayload, CANCEL_COMMAND,
};
