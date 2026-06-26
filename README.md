# Pixelens

A Linux-native visual search and OCR utility.

## Features

### Implemented
- **Screen Capture**: Select any region of your screen using grim/slurp
- **OCR**: Extract text from captured images using Tesseract
- **AI Integration**: Ask AI about captured content (OpenAI-compatible APIs)
- **Actions**: Copy text, web search, reverse image search, translate
- **CLI**: Full command-line interface with all user-facing commands
- **Daemon**: Background service for handling capture and processing
- **IPC**: Unix domain socket communication between CLI and daemon
- **Configuration**: JSON-based config with environment variable support

### Partial
- **Hotkey Support**: Hotkey parsing implemented, global capture trigger pending

### Planned
- **Global Hotkeys**: System-wide capture trigger (requires compositor integration)
- **Clipboard Integration**: Direct clipboard write for copy action
- **Browser Integration**: Open URLs in default browser for search results

## Architecture

```
pixelens/
├── crates/
│   ├── pixelens-common/     # Shared types and errors
│   ├── pixelens-config/     # Configuration management
│   ├── pixelens-capture/    # Screen capture (grim/slurp)
│   ├── pixelens-ocr/        # OCR (Tesseract)
│   ├── pixelens-actions/    # Action handlers and AI client
│   ├── pixelens-ipc/        # IPC layer (Unix domain sockets)
│   ├── pixelens-hotkey/     # Hotkey parsing and matching
│   ├── pixelens-cli/        # CLI binary (pixelens)
│   └── pixelensd/           # Daemon binary (pixelensd)
└── docs/
```

## Requirements

- Rust 1.77.2+
- grim (Wayland screenshot tool)
- slurp (Wayland region selector)
- tesseract-ocr

## Installation

```bash
cargo install --path crates/pixelens-cli
```

## Usage

```bash
# Capture a region and show extracted text
pixelens grab

# Capture and search the web
pixelens grab --search

# Capture and ask AI
pixelens grab --ai "What is this?"

# Copy text to clipboard
pixelens copy "Hello World"

# Search the web
pixelens search "rust programming"

# Ask AI about text
pixelens ai "Explain this code"

# Translate text
pixelens translate "Hello" --to Spanish

# Reverse image search
pixelens image screenshot.png

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
# Pixelens capture
bind = SUPER SHIFT, S, exec, pixelens grab
bind = SUPER SHIFT, A, exec, pixelens grab --ai "What is this?"
bind = SUPER SHIFT, F, exec, pixelens grab --search
```

### Niri

Add to `~/.config/niri/config.kdl`:

```kdl
binds {
    Mod+Shift+S { spawn "pixelens" "grab"; }
    Mod+Shift+A { spawn "pixelens" "grab" "--ai" "What is this?"; }
    Mod+Shift+F { spawn "pixelens" "grab" "--search"; }
}
```

### Sway

Add to `~/.config/sway/config`:

```
# Pixelens capture
bindsym $mod+Shift+s exec pixelens grab
bindsym $mod+Shift+a exec pixelens grab --ai "What is this?"
bindsym $mod+Shift+f exec pixelens grab --search
```
