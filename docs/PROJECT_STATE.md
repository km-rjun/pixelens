# PROJECT_STATE.md — Pixelens

## Product Purpose

Pixelens lets the user select content visible on screen and immediately copy, search, translate, ask AI about it, or perform another contextual action.

## Current Architecture

- `pixelens`: user-facing CLI and daemon controller
- `pixelensd`: background daemon
- `pixelens-core`: capture, OCR, actions, configuration, IPC, menu, upload, search, and shared logic

## Current Working Functionality

- Wayland region selection through `slurp`
- Screenshot capture through `grim`
- OCR through Tesseract
- CLI-to-daemon IPC over Unix domain sockets
- Clean capture cancellation
- `pixelens grab` showing action menu after OCR
- `pixelens copy` copying OCR text via `wl-copy`
- `pixelens search` returning search URL and opening browser
- `pixelens ai` sending text and image to configured AI provider (if model supports vision)
- `pixelens translate` translating OCR text
- `pixelens image` saving image locally and opening Google Lens upload page
- Menu backends: fuzzel, wofi, stdin (auto-detected)
- Daemon start and status behavior verified
- Vision model detection for AI image input
- Quota exhaustion error reporting (insufficient_quota)
- CI passing on GitHub Actions

## Current Command Semantics

All commands are selection-first. Typed positional text is not the normal input model.

| Command | Behavior |
|---------|----------|
| `pixelens grab` | Select region, OCR, show action menu |
| `pixelens copy` | Select region, OCR, copy to clipboard |
| `pixelens search` | Select region, OCR, return search URL |
| `pixelens ai` | Select region, OCR, send text+image to AI (if model supports vision) |
| `pixelens translate` | Select region, OCR, translate (optional `--to`) |
| `pixelens image` | Select region, save image, open Google Lens upload page |
| `pixelens daemon start` | Start pixelensd if not running |
| `pixelens daemon status` | Check daemon via IPC |
| `pixelens daemon stop` | Graceful shutdown via IPC |
| `pixelens config` | Show or set configuration |

## Menu System

`pixelens grab` shows a menu after OCR with keyboard shortcuts:
- `[C] Copy` - Copy text to clipboard
- `[S] Search` - Search the web
- `[A] Ask AI` - Send to AI
- `[T] Translate` - Translate text
- `[Esc] Cancel` - Exit without action

Supported backends (auto-detected or configured):
- `fuzzel` - Wayland-native launcher
- `wofi` - Wayland application launcher
- `stdin` - Fallback for terminal/non-GUI environments

Configure via `menu_backend` in config (default: "auto").

## Known Limitations

- Automatic reverse image search not implemented (opens upload page only)
- Compositor keybindings currently provide the reliable Wayland trigger mechanism
- OCR may preserve layout artifacts from Tesseract

## Next Milestone

Polish Ask AI behavior: improve prompt construction, add default prompts, and verify image input works with vision models.

See `docs/ROADMAP.md` for full roadmap.

## Non-Goals for Next Milestone

- No new frontend framework
- No broad OCR tuning
- No macOS implementation
- No unrelated refactoring
