# Pixelens

Linux-native visual text extraction utility.

Press a hotkey → select a region → text is on your clipboard. That's it.

Pixelens is a lightweight, keyboard-driven alternative to screenshot tools like Flameshot and Grim, but focused on **text extraction**, not screenshots. No menus, no AI prompts, no cloud, no accounts.

## Goals

The fastest workflow must be:

```
Hotkey → Select Region → Text Copied
```

The user must never be forced through menus, confirmations, AI prompts, account creation, cloud services, or provider selection.

## Status

**v0.1.0 — Project scaffold.** Implementation proceeds milestone-by-milestone per `PRD.md` (see `docs/`).

| Milestone | Description | Status |
|---|---|---|
| 1 | Project Setup (workspace, crate stubs, CI) | in progress |
| 2 | Display Server Detection | pending |
| 3 | Capture (Wayland) | pending |
| 4 | Capture (X11) | pending |
| 5 | OCR (Tesseract) | pending |
| 6 | IPC (Unix socket) | pending |
| 7 | Clipboard & Notifications | pending |
| 8 | Configuration | pending |
| 9 | Tray | pending |
| 10 | Packaging (AUR, releases) | pending |
| 11 | Testing & Release | pending |

## Architecture

Two binaries share a workspace of focused crates:

| Crate | Purpose |
|---|---|
| `pixelens-daemon` | `pixelensd` — background service: display detection, capture, OCR, tray, IPC |
| `pixelens-cli` | `pixelens` — thin CLI client speaking to the daemon over a Unix socket |
| `pixelens-core` | Shared types, error enum, base traits |
| `pixelens-capture` | `CaptureProvider` trait + Wayland / X11 implementations |
| `pixelens-ocr` | `OcrEngine` trait + Tesseract implementation |
| `pixelens-overlay` | Region selection overlay (layer-shell / XCB) |
| `pixelens-notify` | Notification abstraction (libnotify / portal) |
| `pixelens-config` | `config.toml` parsing + `pixelens config` commands |
| `pixelens-ipc` | Length-prefixed JSON over Unix domain socket |

## Build

Requires Rust 1.75+ and (for runtime) the `tesseract` binary.

```bash
cargo build --release
```

The two binaries land in `target/release/`:

- `target/release/pixelensd` — the daemon
- `target/release/pixelens`  — the CLI

## Usage

```bash
pixelens                 # show help
pixelens grab            # one-shot capture: select region, copy text
pixelens daemon          # start the daemon in foreground
pixelens status          # show daemon status
pixelens stop            # stop the daemon
pixelens config          # manage configuration
```

## Supported Platforms

- **Display servers:** Wayland, X11
- **Compositors / DEs:** Hyprland, Sway, i3, DWM, GNOME, KDE Plasma

## License

MIT — see [LICENSE](LICENSE).
