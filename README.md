# Pixelens

A Linux-native visual search and OCR utility.

## Status

| Feature | Status |
|---------|--------|
| Screen capture (grim/slurp) | Working |
| OCR (Tesseract) | Working |
| Action menu after capture | Working |
| Clipboard copy (wl-copy) | Working |
| Web search (xdg-open) | Working |
| AI integration | Working |
| Translate | Working |
| Daemon with IPC | Working |
| Reverse image search | MVP (save + upload page) |
| X11 support | Not implemented |

## Architecture

```
pixelens/
├── crates/
│   ├── pixelens/           # CLI binary
│   ├── pixelensd/          # Daemon binary (single source of truth)
│   └── pixelens-core/      # Core library (config, capture, OCR, actions, IPC, menu, upload, search)
├── scripts/
│   ├── install.sh          # Local installation script
│   └── uninstall.sh        # Uninstall script
├── packaging/
│   └── systemd/
│       └── pixelensd.service  # Systemd user service
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
- Menu backend: fuzzel, wofi, or stdin fallback

## Installation

### Quick install (recommended)

```bash
./scripts/install.sh
```

This will:
- Build release binaries
- Install `pixelens` and `pixelensd` to `~/.local/bin`
- Create config and cache directories

### Manual install

```bash
cargo install --path crates/pixelens
cargo install --path crates/pixelensd
```

### Uninstall

```bash
./scripts/uninstall.sh
```

This removes binaries from `~/.local/bin` but preserves your config.

## Usage

All commands select a screen region first, then act on the captured content.

```bash
# Select a region, OCR it, and choose an action from the menu
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

# Save image and open Google Lens upload page
pixelens image

# Start the daemon
pixelens daemon start

# Check daemon status
pixelens daemon status

# Stop the daemon
pixelens daemon stop

# Show configuration
pixelens config

# Set configuration
pixelens config --endpoint https://api.openai.com/v1 --model gpt-4o

# Show version
pixelens version
```

## Daemon Service

Pixelens can run as a systemd user service:

```bash
# Install the service file
cp packaging/systemd/pixelensd.service ~/.config/systemd/user/

# Reload systemd
systemctl --user daemon-reload

# Enable and start the service
systemctl --user enable --now pixelensd

# Check status
systemctl --user status pixelensd
```

## Configuration

Configuration is stored at `~/.config/pixelens/config.json`:

```json
{
  "api_endpoint": "https://api.openai.com/v1",
  "model": "gpt-4o",
  "ocr_language": "eng",
  "menu_backend": "auto",
  "reverse_image_provider": "google_lens"
}
```

### Menu Backends

The action menu uses one of these backends (auto-detected by default):

- **fuzzel** - Wayland-native launcher (recommended)
- **wofi** - Wayland application launcher
- **stdin** - Terminal fallback for non-GUI environments

Set `menu_backend` in config to force a specific backend.

### Reverse Image Search

`pixelens image` saves the captured screenshot locally and opens the Google Lens upload page.

**Privacy note**: Screenshots may contain private data. Automatic upload to external services is disabled by default. To enable automatic upload, configure `image_upload_provider` in the config file.

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
