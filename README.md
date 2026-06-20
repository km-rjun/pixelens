Pixelens - Linux-native visual text extraction

## Overview
Pixelens is a CLI tool that leverages Wayland's IPC for screen capture and text extraction.
It provides a daemon (`pixelensd`) and a client (`pixelens`) to interact with the system.

## Architecture

Pixelens uses Wayland's IPC to create a local pipeline that handles screen capture and text extraction. The workflow consists of:

1. **Daemon (`pixelensd`)** — binds a Unix socket and runs in the background
2. **Screen Capturer (`slurp`)** — selects an area and exits with geometry
3. **Capture Engine (`grim`)** — writes the captured area as a PNG file
4. **Client (`pixelens`)** — interacts with the daemon and processes the file path

The IPC is implemented via `pixelens-ipc`, which handles the full pipeline. This enables the daemon to communicate with the CLI and report status, errors, or capture paths in a structured manner.

## Installation

### Dependencies
- `slurp` for region selection
- `grim` for capturing and saving as PNG
- `pixelensd` (daemon) and `pixelens` (client)

### Building
1. Ensure `Cargo.toml` is set up correctly:

```
[dependencies]
anyhow = { version = "1.0", features = ["std"] }
serde_json = { version = "1.0", features = ["std"] }
thiserror = { version = "1.0", features = ["std"] }
tracing = { version = "0.1", features = ["std"] }
tracing-subscriber = { version = "0.1", features = ["std"] }
tokio = { version = "1.0", features = ["rt