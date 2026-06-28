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

## Prerequisites

Before using Pixelens, ensure the following are installed:

| Tool | Purpose | Install (Arch) |
|------|---------|----------------|
| `grim` | Wayland screenshot capture | `pacman -S grim` |
| `slurp` | Wayland region selector | `pacman -S slurp` |
| `tesseract` | OCR engine | `pacman -S tesseract` |
| `wl-clipboard` | Clipboard integration (`wl-copy`) | `pacman -S wl-clipboard` |
| `fuzzel` or `wofi` | Action menu backend | `pacman -S fuzzel` or `pacman -S wofi` |

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

### Verify installation

```bash
pixelens version
pixelensd --version
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

## Compositor Keybindings

Since global hotkeys are not reliably supported across Wayland compositors, Pixelens uses compositor-level keybindings to trigger capture.

### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```
# Pixelens keybindings
bind = SUPER SHIFT, S, exec, pixelens grab
bind = SUPER SHIFT, C, exec, pixelens copy
bind = SUPER SHIFT, F, exec, pixelens search
bind = SUPER SHIFT, A, exec, pixelens ai
bind = SUPER SHIFT, T, exec, pixelens translate
bind = SUPER SHIFT, I, exec, pixelens image
```

### Niri

Add to `~/.config/niri/config.kdl`:

```kdl
binds {
    Mod+Shift+S { spawn "pixelens" "grab"; }
    Mod+Shift+C { spawn "pixelens" "copy"; }
    Mod+Shift+F { spawn "pixelens" "search"; }
    Mod+Shift+A { spawn "pixelens" "ai"; }
    Mod+Shift+T { spawn "pixelens" "translate"; }
    Mod+Shift+I { spawn "pixelens" "image"; }
}
```

### Sway

Add to `~/.config/sway/config`:

```
# Pixelens keybindings
bindsym $mod+Shift+s exec pixelens grab
bindsym $mod+Shift+c exec pixelens copy
bindsym $mod+Shift+f exec pixelens search
bindsym $mod+Shift+a exec pixelens ai
bindsym $mod+Shift+t exec pixelens translate
bindsym $mod+Shift+i exec pixelens image
```

After adding keybindings, reload your compositor configuration.

## Troubleshooting

### "command not found: pixelens"

- Ensure `~/.local/bin` is in your PATH
- Run `source ~/.bashrc` or `source ~/.zshrc` after installation
- Verify binary exists: `ls ~/.local/bin/pixelens`

### "Daemon not running"

- Start the daemon: `pixelens daemon start`
- Or via systemd: `systemctl --user start pixelensd`
- Check status: `pixelens daemon status`

### "slurp does not open"

- Verify slurp is installed: `which slurp`
- Verify you are in a Wayland session
- Check compositor is running: `echo $WAYLAND_DISPLAY`

### "OCR returns bad text"

- Verify tesseract is installed: `tesseract --version`
- Check language support: `tesseract --list-langs`
- Ensure captured image has sufficient contrast

### "Action menu does not open"

- Verify menu backend is installed: `which fuzzel` or `which wofi`
- Check menu_backend setting: `pixelens config`
- Try setting `menu_backend` to `stdin` as fallback

### "Browser does not open"

- Verify xdg-open is available: `which xdg-open`
- Ensure a default browser is configured
- Check `xdg-settings get default-web-browser`

### "Image search only opens upload page"

This is expected default behavior. To enable automatic upload:
1. Configure `image_upload_provider` in `~/.config/pixelens/config.json`
2. Set `image_upload_url` to your upload endpoint
3. Note: Automatic upload sends screenshots to external services

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
