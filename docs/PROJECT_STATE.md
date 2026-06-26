# PROJECT_STATE.md — Pixelens

## Product Purpose

Pixelens lets the user select content visible on screen and immediately copy, search, translate, ask AI about it, or perform another contextual action.

## Current Architecture

- `pixelens`: user-facing CLI and daemon controller
- `pixelensd`: background daemon
- `pixelens-core`: capture, OCR, actions, configuration, IPC, and shared logic

## Current Working Functionality

- Wayland region selection through `slurp`
- Screenshot capture through `grim`
- OCR through Tesseract
- CLI-to-daemon IPC over Unix domain sockets
- Clean capture cancellation
- `pixelens grab` returning OCR text without changing the clipboard
- `pixelens copy` copying OCR text via `wl-copy`
- Daemon start and status behavior verified
- CI passing on GitHub Actions

## Current Command Semantics

All commands are selection-first. Typed positional text is not the normal input model.

| Command | Behavior |
|---------|----------|
| `pixelens grab` | Select region, OCR, return text |
| `pixelens copy` | Select region, OCR, copy to clipboard |
| `pixelens search` | Select region, OCR, return search URL |
| `pixelens ai` | Select region, OCR, send to AI (optional `--prompt`) |
| `pixelens translate` | Select region, OCR, translate (optional `--to`) |
| `pixelens image` | Not implemented (returns error) |
| `pixelens daemon start` | Start pixelensd if not running |
| `pixelens daemon status` | Check daemon via IPC |
| `pixelens daemon stop` | Graceful shutdown via IPC |
| `pixelens config` | Show or set configuration |

## Known Limitations

- Action menu not implemented yet
- Reverse-image search incomplete (returns "not implemented")
- Compositor keybindings currently provide the reliable Wayland trigger mechanism
- OCR may preserve layout artifacts from Tesseract
- Visual input to AI currently passes only OCR text, not the image

## Next Milestone

Implement the minimal action menu invoked by `pixelens grab`.

## Non-Goals for Next Milestone

- No new frontend framework
- No broad OCR tuning
- No macOS implementation
- No reverse-image upload implementation
- No unrelated refactoring
