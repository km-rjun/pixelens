# 03 — Milestones (M1–M11)

Status legend: ✅ done · 🟡 partial · ❌ not started · ⚠️ regressed

Canonical definitions are in `/root/pixelens-v1.md` §"v1 Milestones". Status
below is **verified against the actual tree and git log**, not assumed.

## M1 — Project Setup — ✅ (mostly)
- [x] Cargo workspace (`Cargo.toml`, 9 members, edition 2021, rust 1.75)
- [x] All crate stubs with correct dependency graph
- [x] `cargo build` passes end-to-end *(was true; currently RED due to CLI
      regression in M-cli — see 08)*
- [x] CI configured (`.github/workflows`, see `9422191`)
- [x] Pushed to GitHub

## M2 — Display Server Detection — ✅
- [x] Detect Wayland vs X11 at runtime (`pixelens-capture::detector`)
- [x] Route stored in daemon state (`DaemonState.display`)
- [x] Error on unknown environment (`PixelensError::NoDisplayServer`)

## M3 — Capture (Wayland) — 🟡 partial
- [x] v1-Wayland path shipped as `slurp`+`grim` `GrabPipeline`
- [ ] `zwlr-layer-shell-v1` overlay (the long-term native path is NOT done)
- [ ] `zwlr-screencopy-manager-v1` capture (native, not done)
- [ ] `xdg-desktop-portal` fallback for GNOME (not done)
> Note: the PRD reserves the native wlr path for later; the v1-shipped path is
> slurp+grim by design. The `CaptureProvider`/`WaylandCaptureProvider` stubs
> exist but are not wired into the grab flow yet.

## M4 — Capture (X11) — ❌
- [ ] XCB overlay — stub only (`pixelens-capture::x11`)
- [ ] Rectangle selection — stub only
- [ ] Screen capture — stub only

## M5 — OCR — ❌
- [ ] Tesseract dependency validation at startup — NOT in daemon startup yet
- [ ] `TesseractOcrEngine` with warm init — stub only (`pixelens-ocr`)
- [ ] AT-SPI native extraction attempt with OCR fallback — not started

## M6 — IPC — ✅ (substantially)
- [x] Length-prefixed JSON socket protocol (`pixelens-ipc::codec`)
- [x] Daemon IPC server (`pixelens-daemon::ipc`)
- [x] CLI IPC client — ⚠️ **regressed** (see 08; `6dfb60a` had it, `b6d5d33`
      broke it)
- [x] Request ID and cancel support (protocol + dispatch wired for `cancel`
      command, though handler currently returns "not implemented")

## M7 — Clipboard and Notifications — ❌
- [ ] Clipboard write — not implemented
- [ ] `libnotify` notification sender — stub only (`pixelens-notify`)
- [ ] All notification cases covered — not started

## M8 — Configuration — ❌
- [ ] `config.toml` parsing with defaults — stub only (`pixelens-config`)
- [ ] `pixelens config` CLI commands — not implemented
- [ ] Autostart via `.desktop` file — not started

## M9 — Tray — ❌
- [ ] System tray icon — not started
- [ ] Capture Text / Settings / Quit menu — not started

## M10 — Packaging — ❌
- [ ] AUR package — not started
- [ ] Release binaries (x86_64, aarch64) — not started

## M11 — Testing and Release — 🟡 partial
- [x] Integration tests dir + conventions (`tests/README.md`)
- [x] Some unit tests in capture/ipc (`pipeline.rs`, `protocol.rs`)
- [ ] Integration tests across Wayland and X11 — not started
- [ ] Performance benchmarks against targets — not started
- [ ] Documentation — partial (`docs/`, now this plan folder)
- [ ] v1.0 release — not started

## Blocker before anything else
**The CLI is broken (M6 client side) and the workspace does not build.**
Resolving `08-crate-pixelens-cli.md` is priority #1 — see `10-progress.md`.
