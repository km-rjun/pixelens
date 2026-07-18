Pixelens - Linux-native visual text extraction

## Overview
Pixelens is a CLI tool that leverages Wayland's IPC for screen capture and text extraction.

## Architecture
Pixelens uses Wayland's IPC to create a local pipeline that handles screen capture and text extraction. The workflow consists of:

1. **Daemon (`pixelensd`)** — binds a Unix socket and runs in the background
2. **Screen Capturer (`slurp`)** — selects an area and exits with geometry
3. **Capture Engine (`grim`)** — writes the captured area as a PNG file
4. **Client (`pixelens`)** — interacts with the daemon and processes the file path

The IPC is implemented via `pixelens-ipc`, which handles the full pipeline. This enables the daemon to communicate with the CLI and report status, errors, or capture paths in a structured manner.

## Implementation

The `pixelens grab` workflow is implemented through the following components:
- `pixelensd` daemon for Unix socket IPC
- `slurp` for region selection
- `grim` for capture and PNG output
- `pixelens` CLI for interaction

The implementation includes:
- Wayland detection (`wayland::is_wayland()`)
- Area selection with slurp
- Slurp output parsing
- Capture cancellation handling (`--cancel`)
- Screenshot saving to `/tmp/screenshot.png`