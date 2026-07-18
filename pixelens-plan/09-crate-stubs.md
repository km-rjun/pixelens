# 09 — Crate Stubs: ocr / overlay / notify / config

These four crates are declared in the workspace and compile as stubs, but their
v1 functionality is **not implemented**. They are placeholders for M4–M9.

## pixelens-ocr — ❌ stub (M5)
- Declares `OcrEngine` trait (from `pixelens-core::traits`) and a
  `TesseractOcrEngine` placeholder.
- **Not** wired into the daemon startup; Tesseract is **not** validated or warmed.
- PRD M5 requires: Tesseract validated at startup, warm-init `TesseractOcrEngine`,
  AT-SPI native extraction attempted first with OCR fallback.
- Blocked on M5; also logically requires the daemon to own OCR (see 07).

## pixelens-overlay — ❌ stub (M3 native / M9)
- Intended for the capture-time selection overlay (wlr-layer-shell on Wayland,
  XCB on X11). The v1-Wayland path currently uses `slurp` instead, so this is
  unused for the shipped path. Native overlay work is deferred (PRD M3 native
  path / M9 tray is separate).

## pixelens-notify — ❌ stub (M7)
- Intended notification abstraction (libnotify / portal). Not wired in.
- PRD M7 requires: clipboard write + `libnotify` sender + all four notification
  cases (text copied, no text found, Tesseract missing, daemon not running),
  all auto-dismiss, no modals.

## pixelens-config — ❌ stub (M8)
- Intended `config.toml` parsing + `pixelens config` CLI commands + autostart
  `.desktop` management.
- PRD M8 requires: default config written on first run, `config get/path/set`
  with dot-notation keys, autostart enable/disable writing/removing
  `~/.config/autostart/pixelens.desktop`.

## State: 🟡 all four compile, none implement v1 behaviour
They must remain compiling (so `cargo build` stays green) but contain no
production logic yet. Do not delete them — they are part of the declared
workspace and the PRD repository structure.

## Work remaining
- M4 (X11 capture) touches `pixelens-capture::x11` and may use `pixelens-overlay`.
- M5 implements `pixelens-ocr` + daemon wiring.
- M7 implements `pixelens-notify` + clipboard (clipboard may live in the daemon
  directly; confirm against PRD when starting M7).
- M8 implements `pixelens-config` + `pixelens config` CLI subcommands.
