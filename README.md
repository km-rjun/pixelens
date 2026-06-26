# Pixelens

A Linux-native visual search and OCR utility.

## Features

- **Screen Capture**: Select any region of your screen using grim/slurp
- **OCR**: Extract text from captured images using Tesseract
- **Clipboard**: Copy extracted text to clipboard via wl-copy
- **AI Integration**: Ask AI about captured content (OpenAI-compatible APIs)
- **Browser**: Open search results in default browser
- **CLI**: Selection-first commands for all operations
- **Daemon**: Background service for handling capture and processing
- **IPC**: Unix domain socket communication between CLI and daemon
- **Configuration**: JSON-based config with environment variable support

## Architecture

```
pixelens/
├── crates/
│   ├── pixelens/           # CLI binary
│   ├── pixelensd/          # Daemon binary
│   └── pixelens-core/      # Core library (config, capture, OCR, actions, IPC, hotkey)
├── docs/
├── Cargo.toml              # Workspace root
└── README.md
```

## Requirements

- Rust 1.77.2+
- grim (Wayland screenshot tool)
- slurp (Wayland region selector)
- tesseract-ocr
- wl-clipboard (for `wl-copy`)

## Installation

```bash
cargo install --path crates/pixelens
```

## Usage

All commands select a screen region first, then act on the captured content.

```bash
# Select a region, OCR it, and print the text
pixelens grab

# Select a region, OCR it, copy text to clipboard
pixelens copy

# Select a region, OCR it, search the web
pixelens search

# Select a region, OCR it, ask AI about it
pixelens ai
pixelens ai --prompt "What is happening here?"

# Select a region, OCR it, translate the text
pixelens translate --to Spanish
pixelens translate --to French

# Select a region, perform reverse image search
pixelens image

# Start the daemon
pixelens daemon

# Show daemon status
pixelens status

# Stop the daemon
pixelens stop

# Show configuration
pixelens config

# Set configuration
pixelens config --endpoint https://api.openai.com/v1 --model gpt-4o

# Show version
pixelens version
```

## Configuration

Configuration is stored at `~/.config/pixelens/config.json`:

```json
{
  "api_endpoint": "https://api.openai.com/v1",
  "model": "gpt-4o",
  "ocr_language": "eng",
  "hotkey": "Ctrl+Shift+C"
}
```

### API Key

API keys can be provided via:
1. Environment variable: `PIXELENS_API_KEY=sk-...`
2. Configuration file (not recommended for production)

The environment variable takes precedence and is not saved to the config file.

## Development

```bash
# Run tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build
cargo build --workspace
```

## License

MIT

## Compositor Keybindings

Since global hotkeys are not reliably supported across Wayland compositors, Pixelens uses compositor-level keybindings to trigger capture.

### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```
bind = SUPER SHIFT, S, exec, pixelens grab
bind = SUPER SHIFT, C, exec, pixelens copy
bind = SUPER SHIFT, F, exec, pixelens search
bind = SUPER SHIFT, A, exec, pixelens ai
```

### Niri

Add to `~/.config/niri/config.kdl`:

```kdl
binds {
    Mod+Shift+S { spawn "pixelens" "grab"; }
    Mod+Shift+C { spawn "pixelens" "copy"; }
    Mod+Shift+F { spawn "pixelens" "search"; }
    Mod+Shift+A { spawn "pixelens" "ai"; }
}
```

### Sway

Add to `~/.config/sway/config`:

```
bindsym $mod+Shift+s exec pixelens grab
bindsym $mod+Shift+c exec pixelens copy
bindsym $mod+Shift+f exec pixelens search
bindsym $mod+Shift+a exec pixelens ai
```
