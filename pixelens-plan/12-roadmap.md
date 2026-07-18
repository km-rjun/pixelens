# 12 — Roadmap (where the project is headed)

This file states the absolute destination and the path to it. It is derived
from the PRD's success criteria and the 11 milestones. Read it when deciding
what to build next.

## Absolute destination (v1.0)

A Linux-native utility where a user presses a hotkey, selects a screen region,
and the extracted text lands on their clipboard in under 2 seconds — with zero
menus, confirmations, cloud, or accounts. That is the whole point. Everything
below serves only that.

## The critical path (what actually delivers the success criterion)

Today the daemon can capture a region to a PNG via slurp+grim, but it does NOT
yet turn that PNG into text. The missing links, in dependency order:

1. **OCR (M5)** — validate Tesseract at daemon startup, warm-init
   `TesseractOcrEngine`, attempt AT-SPI first then fall back to OCR. **Without
   this, "text copied" is impossible.**
2. **Clipboard + Notify (M7)** — on a successful capture, run OCR, write the
   text to the clipboard (unless empty → `No text found in selection.`), fire the
   `✓ Text copied to clipboard` libnotify notification. This is the moment the
   PRD success criterion becomes real.
3. **CLI restore (this session's blocker)** — the CLI must reach the daemon
   over IPC so a human can actually trigger a grab and read the result.

Once 1–3 exist, the core loop `hotkey → select → text copied` works end-to-end,
even if invoked via CLI rather than a hotkey. That is the minimum shippable v1
heart.

## Remaining milestones after the critical path
- **M4 (X11)** — real XCB overlay + capture in `pixelens-capture::x11`; route
  via `CaptureBackend`.
- **M8 (config)** — `config.toml` defaults, `pixelens config get/path/set`,
  autostart `.desktop`.
- **M9 (tray)** — Capture Text / Settings / Quit menu (optional; Pixelens
  must work without it).
- **M3 native polish** — optional wlr-layer-shell + wlr-screencopy and
  xdg-desktop-portal fallback (the v1-shipped slurp+grim path already satisfies
  the v1 capture requirement by design).
- **M10 (packaging)** — AUR package, release binaries (x86_64, aarch64).
- **M11 (testing & release)** — integration tests across Wayland/X11, perf
  benchmarks vs the PRD table, documentation, **v1.0 tag**.

## Hard constraints that bound all of the above (from PRD)
- No DBus in v1; CLI talks to daemon only over the Unix socket.
- No AI, search, translate, cloud, accounts, sync, history, plugins.
- `show_preview` defaults `false`; confirmation is opt-in only.
- Display detection runs first at daemon startup; no component branches on
  display type independently.
- Small, single-logical-change commits; never batch unrelated work.
- Build + tests green before a milestone is called done.

## Suggested execution order
1. Fix CLI (`08`), get build green. ← do this now.
2. M5 OCR + daemon wiring.
3. M7 clipboard + notify → core loop works.
4. M4 X11.
5. M8 config.
6. M9 tray (optional).
7. M10 packaging + M11 testing/release → v1.0.

## Definition of "we are done"
All 11 milestones complete, `cargo build` + `cargo test` green, the
`hotkey → select → text copied` loop demonstrably works on a real Wayland
session within the PRD's timing targets, and a v1.0 release is tagged and
pushed.
