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

## M5 — OCR — 🟡 partial (engine + daemon wiring done; live QA not exercised)
- [x] Tesseract dependency validation at startup — `TesseractOcrEngine::new()`
      runs `tesseract --version`; returns `OcrError::EngineMissing` if absent.
      Wired into daemon `run()`: missing tesseract logs a warn, **not fatal** —
      capture still works, grabs return empty `text`.
- [x] `TesseractOcrEngine` with warm init — implemented in `pixelens-ocr/src/lib.rs`:
      `new()`, `with_config(lang, psm)`, `extract_from_path(&Path)`, `extract_text(&CaptureImage)`
      (encodes `CaptureImage` to 24-bit BMP, no extra crate deps).
- [x] Daemon `handle_grab` runs OCR on the captured PNG and attaches `text` to
      `GrabResponsePayload` (new `#[serde(default)]` field in `pixelens-ipc`).
- [x] OCR failure is non-fatal: capture returns path + empty text.
- [ ] AT-SPI native extraction attempt with OCR fallback — not started (post-v1).
- [ ] Live OCR end-to-end QA — **blocked**: headless VM has no display + no real
      screenshot; the unit tests cover sanitize + error paths only (no tesseract
      binary required at test time, though tesseract IS installed here).
> Tesseract 5.5.0 is installed on this host; the engine *would* run live, but a
> real Wayland/X11 screenshot cannot be produced headless. Live OCR QA (real
> image → text) is deferred to a session with a display.

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

---

# Upgrade Milestones (post-v1.0)

These extend the product without violating the PRD core: hotkey → select →
text copied in <2s, **no menus / no cloud / no AI**. Each has a detailed design
doc and its own QA checkpoint + safety gate.

Status legend: 📋 planned · 🚧 in progress · ✅ done

## UM1 — Hotkey Daemon — 🚧 in progress (design: `14-upgrade-m1-hotkey.md`)
- [x] New crate `pixelens-keyhook` (Wayland evdev + X11 x11rb backends)
- [x] `pixelens hotkey enable|disable|status` CLI
- [x] systemd `--user` unit generated + installed on enable
- [ ] Config seed: `[hotkey] enabled, combo` (deferred to M8 config file I/O)
- [x] Hotkey press → daemon `Grab` over existing Unix socket
- [ ] Manual QA on real Wayland + X11 session (blocked: headless env)

## UM2 — Windows Support — 📋 planned (design: `15-upgrade-m2-windows.md`)
- [ ] `cfg(windows)` capture pipeline (WinRT / GDI+ fallback)
- [ ] Named-pipe IPC transport (Unix socket → `\\.\pipe\pixelens`)
- [ ] `RegisterHotKey` Win+Shift+S → replaces Snipping Tool
- [ ] `arboard` clipboard + `winrt-notification` toast
- [ ] WiX/winget packaging draft

## UM3 — Systemd Service + Autostart — 📋 planned (in `13-roadmap-upgrades.md`)
- [ ] `pixelens daemon install|uninstall`; socket-activated user unit
- [ ] `config autostart` toggle

## UM4 — Grab UI / Actions Popup — 📋 planned (design: `16-upgrade-m4-grab-ui.md`)
- [ ] New crate `pixelens-gui` (egui HUD, keyboard-first)
- [ ] Hotkey+Space opens transient HUD (g / r / p actions)
- [ ] IPC: `Command::Redetect`, `Command::SetPreview`
- [ ] Default grab path unchanged when HUD disabled

## UM5–UM8 — Portal capture, multi-display, on-demand mode, OCR tuning
See `13-roadmap-upgrades.md` for scope. Not yet split into design docs.

---

## Execution order (recommended)
1. **UM1** (this session) → 2. UM3 → 3. UM2 (Windows) → 4. UM4 (HUD) →
   5. UM5–UM8.
