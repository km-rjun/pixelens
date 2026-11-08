# 07 — Crate: pixelens-daemon

**Role**: the background daemon (`pixelensd`). Owns display detection, capture
pipeline, and the IPC server.
**Layout**:
- `pixelens-daemon/src/main.rs` — 12-line bin; builds a multi-thread tokio
  runtime and calls `pixelens_daemon::run()`.
- `pixelens-daemon/src/lib.rs` — `run()` + `init_tracing()`.
- `pixelens-daemon/src/dispatch.rs` — `Dispatcher` mapping `Command` → handler.
- `pixelens-daemon/src/ipc.rs` — Unix socket server.
- `pixelens-daemon/src/state.rs` — `DaemonState`.
**Depends on**: core, ipc, capture (+ eventually ocr, overlay, notify, config)

## lib.rs — ✅
- `init_tracing()` idempotent; `run()`:
  1. detect display server (fatal if none),
  2. build `GrabPipeline` (warns + continues if slurp/grim missing),
  3. bind IPC socket (fatal if it fails),
  4. build `DaemonState` + `Dispatcher`, serve forever.

## state.rs — ✅
- `DaemonState { display: DisplayServer, pipeline: Option<GrabPipeline> }`.
- `pipeline` is `None` when slurp/grim absent; `grab` then returns a clear
  `MissingTool` error and the daemon stays up for other commands.

## ipc.rs — ✅
- `socket_path()` honours `$XDG_RUNTIME_DIR`, falls back to
  `/tmp/pixelens-$UID.sock` via a one-off `getuid` FFI call (no `nix` dep).
- `bind()` removes stale socket, binds `UnixListener`.
- `serve()` accept loop; each connection → `read_frame` → `dispatch` →
  `write_response`. One request/response per short-lived connection.

## dispatch.rs — ✅
- `Grab` → `handle_grab`: runs pipeline on a blocking thread
  (`block_in_place`), maps `GrabOutcome::{Captured,Cancelled}` to
  `IpcResponse::{ok, cancelled}`, maps errors to `error`.
- `Status` → returns version + display + `pipeline_ready`.
- `Stop` → schedules `std::process::exit(0)` after replying.
- `ConfigGet/ConfigSet/Cancel` → `handle_not_implemented` (returns
  `"...is not yet implemented in this build"` with `ResponseStatus::Error`).

## State: ✅ solid for M1/M2/M3(v1-path)/M6
The daemon correctly detects display, serves IPC, and performs grabs via
slurp+grim. It does NOT yet do OCR, clipboard, notify, config, or tray.

## Work remaining (future milestones)
- M5: validate Tesseract at startup, keep `OcrEngine` warm, run OCR on the
  captured file, then clipboard + notify.
- M6 polish: implement `Cancel` against an active session.
- M7/M8/M9: clipboard, notify, config, tray integration here.
