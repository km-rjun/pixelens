# Pixelens

**Linux-native visual text extraction.** Select any region on your screen and have
the text copied to your clipboard in under 2 seconds — no cloud, no accounts,
no menus.

## Quick start

1. Install dependencies (Wayland):
   - `slurp` — region selection
   - `grim` — screenshot capture  
   - `tesseract` — OCR (required at daemon startup)

2. Build:
   ```bash
   git clone https://github.com/km-rjun/pixelens
   cd pixelens
   cargo build --release
   ```

3. Run the daemon (one time, or add to autostart):
   ```bash
   ./target/release/pixelensd &
   # or for persistent background: ./target/release/pixelensd --daemon
   ```

4. Grab text:
   ```bash
   ./target/release/pixelens grab
   # or: ./target/release/pixelens copy
   ```

5. (Optional) Bind a global hotkey so you never type the command:
   ```bash
   ./target/release/pixelens hotkey enable
   ```
   Now press **Super+Shift+T** (configurable via `general.hotkey` / the
   `PIXELENS_HOTKEY` env var) anywhere to trigger a grab. On Wayland, the
   listener reads raw input devices, so your user must be in the `input`
   group: `sudo usermod -aG input $USER` then re-login.

The selected region's text is now on your clipboard.

## Dependencies

| Tool | Purpose | Debian/Ubuntu | Arch |
|------|---------|---------------|------|
| slurp | Region selection | `apt install slurp` | `pacman -S slurp` |
| grim  | Screen capture | `apt install grim` | `pacman -S grim` |
| tesseract | OCR engine | `apt install tesseract-ocr` | `pacman -S tesseract` |

All three are required. The daemon checks at startup; if any is missing, it
prints installation instructions and exits.

## Commands

```
pixelens grab, copy   Select region → copy text to clipboard
pixelens status        Show daemon version and display server
pixelens stop          Stop the daemon
pixelens hotkey        Manage global hotkey (enable|disable|status)
pixelens config        Manage settings (see below)
pixelens version       Show version
pixelens help          Show help
```

## Configuration

Config file: `~/.config/pixelens/config.toml`

Default (written on first run):
```toml
[general]
autostart = false
theme = "system"
hotkey = "Super+Shift+T"

[capture]
show_preview = false
```

Only `show_preview = true` enables the optional preview/confirm step. The default
path is zero-friction: select → text → clipboard, no confirmation.

## Architecture

```
    hotkey
      ↓
Selection overlay (slurp → grim)
      ↓
  OCR (Tesseract)
      ↓
Clipboard + notification
```

The daemon (`pixelensd`) owns the capture and OCR subsystems, warmed at startup.
The CLI (`pixelens`) talks to it exclusively over a Unix socket
(`$XDG_RUNTIME_DIR/pixelens.sock`). The capture flow runs `slurp` for region
selection and `grim` for capture. OCR (Tesseract) extracts text, then it's
copied to the clipboard.

## Troubleshooting

**Daemon not running:**
```
pixelensd &
```

**slurp/grim/tesseract missing:**
Install via your package manager (see table above). The daemon startup will show
the appropriate command.

**Clipboard empty after grab:**
The region may contain no detectable text. Or Tesseract may not be installed.

**Hotkey not firing on Wayland:**
The keyhook reads `/dev/input/event*` directly. Your user must be in the
`input` group: `sudo usermod -aG input $USER`, then log out and back in.
Check with `groups`. If the group is missing, `pixelens hotkey status` will
report the listener as inactive and the daemon logs a clear error.

## License

MIT