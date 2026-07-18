# 06 — Crate: pixelens-capture

**Role**: display-server detection + capture backends. Also hosts the v1-Wayland
`slurp`+`grim` pipeline.
**Path**: `pixelens-capture/src/{lib,detector,slurp_grim,pipeline,which,wayland,x11}.rs`
**Depends on**: `pixelens-core`

## Modules

### detector.rs — ✅ M2 done
- `DisplayServer` enum: `Wayland, X11`.
- `detect_display_server()`: Wayland if `$WAYLAND_DISPLAY`, else X11 if
  `$DISPLAY`, else `Err(PixelensError::NoDisplayServer)`. Runs at daemon
  startup; result stored in `DaemonState`.

### which.rs — ✅
- `which(tool) -> Result<PathBuf, WhichError>` — `$PATH` lookup used for the
  slurp/grim dependency probe.

### slurp_grim.rs — ✅ core of v1-Wayland
- `format_geometry(Rect) -> "WxH+X+Y"` (min 1x1 so grim won't reject).
- Traits `RegionSelector` (`select -> Option<Rect>`) and `ScreenCapturer`
  (`capture(region, path)`).
- `SlurpSelector`: shells out to `slurp`; treats empty-stdout + non-zero exit
  as **cancel** (`Ok(None)`).
- `GrimCapturer`: shells out to `grim -g <geom> -o <path>`.
- Well-documented rationale in the module header for why this is *separate* from
  the long-term `CaptureProvider` wlr-screencopy path.

### pipeline.rs — ✅ M3 (v1-Wayland path)
- `GrabOutcome::{Captured{path,region,bytes}, Cancelled}`.
- `GrabError`/`GrabErrorKind { MissingTool, Subprocess, Output }`.
- `GrabPipeline`: upfront `which(slurp)`+`which(grim)` check; `run()` →
  select → allocate temp path under `$TMPDIR/pixelens/` → capture → verify
  non-zero bytes → return `Captured`. Cancel and missing-tool map to clean
  outcomes/errors.
- **Tests** present: `new_succeeds_when_dependencies_present`,
  `cancelled_selector_returns_cancelled_outcome`,
  `captured_selector_writes_file_and_returns_path`,
  `capture_failure_propagates_as_grab_error` (use fakes; skip if slurp/grim
  absent).

### wayland.rs / x11.rs — 🟡 stubs
- `WaylandCaptureProvider` / `X11CaptureProvider` implement `CaptureProvider`
  but are **not wired into the grab flow**. They are the reserved long-term
  native path; `lib.rs` aggregates them into `CaptureBackend`.

### lib.rs — ✅
- Re-exports everything above; defines `CaptureBackend` enum routing to the
  per-display provider (used by nothing yet — the daemon uses `GrabPipeline`
  instead for the v1 path).

## State: ✅ strong for the shipped v1 path, 🟡 partial vs PRD M3
The v1-Wayland `slurp`+`grim` path is real, tested, and used by the daemon.
The *native* wlr-layer-shell / wlr-screencopy path and the X11 path are stubs.

## Work remaining
- (Optional, post-v1) Implement native wlr-screencopy + xdg-desktop-portal
  fallback per PRD M3.
- (M4) Implement real X11/XCB backend in `x11.rs`.
- Wire `CaptureBackend` into the daemon once native backends exist.
