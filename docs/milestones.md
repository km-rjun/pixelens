# Milestones

The PRD defines 11 milestones for v1.0. This file is the working
breakdown — the canonical definitions are in `/root/pixelens-v1.md`.

## M1 — Project Setup *(in progress)*

- [x] Cargo workspace
- [x] All 9 crate stubs with the dependency graph from PRD §"Repository Structure"
- [x] `cargo build` passes end-to-end
- [x] CI configured
- [x] Push to GitHub

## M2 — Display Server Detection

- [ ] Detect Wayland vs X11 at runtime
- [ ] Route stored in daemon state
- [ ] Error on unknown environment

## M3 — Capture (Wayland)

- [ ] `zwlr-layer-shell-v1` overlay
- [ ] Rectangle selection
- [ ] `zwlr-screencopy-manager-v1` capture
- [ ] `xdg-desktop-portal` fallback for GNOME

## M4 — Capture (X11)

- [ ] XCB overlay
- [ ] Rectangle selection
- [ ] Screen capture

## M5 — OCR

- [ ] Tesseract dependency validation at startup
- [ ] `TesseractOcrEngine` with warm init
- [ ] AT-SPI native extraction attempt with OCR fallback

## M6 — IPC

- [ ] Length-prefixed JSON socket protocol
- [ ] Daemon IPC server
- [ ] CLI IPC client
- [ ] Request ID and cancel support

## M7 — Clipboard and Notifications

- [ ] Clipboard write
- [ ] `libnotify` notification sender
- [ ] All notification cases covered

## M8 — Configuration

- [ ] `config.toml` parsing with defaults
- [ ] `pixelens config` CLI commands
- [ ] Autostart via `.desktop` file

## M9 — Tray

- [ ] System tray icon
- [ ] Capture Text / Settings / Quit menu

## M10 — Packaging

- [ ] AUR package
- [ ] Release binaries (x86_64, aarch64)

## M11 — Testing and Release

- [ ] Integration tests across Wayland and X11
- [ ] Performance benchmarks against targets
- [ ] Documentation
- [ ] v1.0 release
