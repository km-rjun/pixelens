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

pub use codec::{read_frame, write_frame, FrameError, MAX_FRAME_SIZE};
pub use protocol::{
    Command, IpcError, IpcRequest, IpcResponse, RequestId, ResponseStatus, CANCEL_COMMAND,
};
