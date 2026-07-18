# 05 — Crate: pixelens-ipc

**Role**: wire protocol + socket framing between CLI and daemon.
**Path**: `pixelens-ipc/src/{lib,protocol,codec}.rs`
**Depends on**: `pixelens-core` (re-exports `Rect` via `From` impls)

## Wire format (PRD §IPC)
```
[4 bytes: u32 big-endian payload length][N bytes: UTF-8 JSON]
```
Max frame size is bounded (`MAX_FRAME_SIZE` in `codec`).

## protocol.rs
- `RequestId = String` (UUIDv4 on the wire).
- `Command` enum: `Grab, Status, Stop, ConfigGet, ConfigSet, Cancel`.
- `ResponseStatus`: `Ok, Error, Cancelled`.
- `IpcRequest { request_id, command, payload: serde_json::Value }`.
- `IpcResponse { request_id, status, payload }` with `ok()` / `error()` /
  `cancelled()` constructors.
- `GrabResponsePayload { path, region: GrabRegion, bytes, captured_at_ms }`.
- `GrabRegion { x, y, width, height }` with `From<pixelens_core::Rect>`.
- `IpcError` enum (invalid frame, too large, io, json).
- **Tests**: `ok_response_carries_serialized_payload`,
  `grab_payload_round_trips`, `error_response_includes_message`,
  `cancelled_response_marks_status` — all present and meaningful.

## codec.rs
- `read_frame`, `write_frame`, `read_response`, `write_response`,
  `FrameError`, `MAX_FRAME_SIZE`.
- Used by both the daemon server (`ipc.rs`) and (intended) the CLI client.

## State: ✅ complete for M6
The protocol and framing are real and unit-tested. The daemon server uses them
(`pixelens-daemon/src/ipc.rs` calls `read_frame`/`write_response`). The **CLI
client side is regressed** — see `08-crate-pixelens-cli.md`.

## Work remaining
- CLI must use `read_response`/`write_frame` again (it was replaced by a
  broken hand-rolled stub).
- `cancel` is a first-class command but the dispatcher returns "not
  implemented" for it — wire `Cancel` to an active session later (M6 polish).
