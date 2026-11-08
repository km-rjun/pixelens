# 02 — Architecture

Top-level flow (from `docs/architecture.md`, reproduced so this folder is
self-contained):

```
                ┌──────────────────────────────┐
   hotkey       │        pixelensd            │
  ─────────►    │  ┌────────────────────────┐  │   ┌──────────────┐
                │  │  display-server det.   │  │   │ pixelens     │
                │  └─────────┬──────────────┘  │   │ (CLI client) │
                │            ▼                 │   └──────┬───────┘
                │  ┌────────────────────────┐  │          │ IPC
                │  │   CaptureProvider      │  │          │ (Unix sock)
                │  │  (Wayland | X11)       │  │          │
                │  └─────────┬──────────────┘  │          ▼
                │            ▼                 │
                │  ┌────────────────────────┐  │
                │  │   Selection overlay    │  │
                │  └─────────┬──────────────┘  │
                │            ▼                 │
                │  ┌────────────────────────┐  │
                │  │   OcrEngine (warm)     │  │
                │  └─────────┬──────────────┘  │
                │            ▼                 │
                │  ┌────────────────────────┐  │
                │  │   clipboard + notify   │  │
                │  └────────────────────────┘  │
                └──────────────────────────────┘
```

## Component responsibilities

- **Display Server Detector** — runs first at startup. Wayland if
  `$WAYLAND_DISPLAY` set, else X11 if `$DISPLAY` set, else fatal error.
  Result stored in `DaemonState` and used to route all capture/overlay.
- **Capture Engine** — `CaptureProvider` trait with `WaylandCaptureProvider`
  and `X11CaptureProvider` long-term impls. v1-Wayland ships a *separate*
  `slurp`+`grim` pipeline (`pixelens-capture::pipeline`) because its
  process/file model differs from the in-process wlr-screencopy path.
- **OCR Engine** — `OcrEngine` trait; v1 impl `TesseractOcrEngine`, warmed
  at startup. AT-SPI native extraction attempted first, OCR fallback.
- **Hotkey Manager**, **Tray Service**, **Config Manager**, **IPC Server** —
  daemon-owned subsystems; only IPC is implemented so far.

## IPC

- Transport: Unix domain socket at `$XDG_RUNTIME_DIR/pixelens.sock`
  (fallback `/tmp/pixelens-$UID.sock`).
- Framing: `[4-byte u32 BE length][N-byte UTF-8 JSON]`.
- Every request carries `request_id` (UUIDv4); responses echo it so `cancel`
  matches the in-flight session.
- See `05-crate-pixelens-ipc.md` and `docs/architecture.md` §IPC.

## Crate dependency graph (lowest → highest)

```
pixelens-core        (types, errors, traits; no deps on siblings)
pixelens-ipc        (protocol + codec; depends on core)
pixelens-capture    (detection + slurp/grim + CaptureProvider; depends on core)
pixelens-ocr        (OcrEngine + Tesseract; depends on core)
pixelens-overlay    (selection UI; depends on core)
pixelens-notify     (notifications; depends on core)
pixelens-config     (config.toml; depends on core)
pixelens-daemon     (bins: pixelensd; lib: pixelens_daemon; depends on all above)
pixelens-cli        (bin: pixelens; depends on ipc, core)
```

## Where the actual code lives (verified paths)

- Daemon lib: `pixelens-daemon/src/{lib,dispatch,ipc,state}.rs`
- Daemon bin: `pixelens-daemon/src/main.rs` (12 lines, thin wrapper)
- Capture: `pixelens-capture/src/{detector,slurp_grim,pipeline,which,wayland,x11,lib}.rs`
- IPC: `pixelens-ipc/src/{protocol,codec,lib}.rs`
- CLI (BROKEN): `pixelens-cli/src/main.rs` — see `08-crate-pixelens-cli.md`

## Key design decisions already made (don't re-litigate without cause)

1. `slurp`+`grim` v1-Wayland path lives *outside* the `CaptureProvider` trait
   (separate `GrabPipeline`) because the process/file model is different from
   the in-process wlr-screencopy path the trait is reserved for.
2. Daemon is split lib + bin so integration tests drive it in-process.
3. `pixelens-ui` from the original PRD structure was split into `pixelens-overlay`
   (capture-time UI) and `pixelens-notify` (fire-and-forget) because they have
   different lifetimes/dependencies.
4. IPC `payload` is `serde_json::Value` in v1; per-command payloads validated
   at the dispatch layer, not centralised in the enum.
