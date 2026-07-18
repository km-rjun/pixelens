# Pixelens

Pixelens is a Linux visual text-extraction utility. You press a hotkey (or run a
command), select a screen region with the cursor, and the grabbed region is handled
by a background daemon. **Current build:** the daemon captures the selected region to
a screenshot file and reports its path. OCR (turning the screenshot into text) and
copying that text to the clipboard are **not yet wired in** — they are the next
milestones. Everything below documents what the code actually does today.

## What works today

| Piece | Status |
|-------|--------|
| Region capture (`slurp` + `grim`) | ✅ working |
| Background daemon + CLI over a Unix socket | ✅ working |
| Global hotkey (systemd user service) | ✅ working (Wayland + X11) |
| OCR via Tesseract | ❌ NOT YET (M5) |
| Copy extracted text to clipboard | ❌ NOT YET (M7) |
| `pixelens config` CLI subcommand | ❌ stub (edit the TOML file directly) |
| `show_preview` / `autostart` / `theme` config keys | ⚠️ parsed, not yet consumed by the daemon |

So `pixelens grab` today **captures a screenshot of the selected region** and prints
the saved file path — it does not yet return OCR text. This README is kept honest to
the code; features marked NOT YET are planned, not present.

## Build from source

Requires the Rust toolchain (edition 2021).

```bash
git clone https://github.com/km-rjun/pixelens
cd pixelens
cargo build --release
```

This produces two binaries in `target/release/`:

- `pixelensd` — the background **daemon** (owns capture + IPC).
- `pixelens` — the **CLI** client (talks to the daemon over a socket).
- `pixelens-keyhook` — the **global hotkey listener** (spawned by `pixelens hotkey enable`).

## Runtime dependencies (REQUIRED)

Pixelens shells out to external tools for capture. These must be on `$PATH`.

| Tool | Purpose | Debian/Ubuntu | Arch (pacman) | Fedora (dnf) |
|------|---------|---------------|---------------|--------------|
| `slurp` | Region selection | `sudo apt install slurp` | `sudo pacman -S slurp` | `sudo dnf install slurp` |
| `grim`  | Screen capture  | `sudo apt install grim`  | `sudo pacman -S grim`  | `sudo dnf install grim`  |

Both are **required** for the current build. The daemon's capture pipeline checks for
them at startup; if either is missing, `pixelens grab` returns an error telling you
which tool to install and restart the daemon.

> **Tesseract / OCR:** `tesseract-ocr` is the planned OCR engine (M5) but is **not
> yet consumed** by the daemon in this build, so it is not required to capture a
> screenshot. Install it ahead of time if you want to be ready:
> `sudo apt install tesseract-ocr` / `sudo pacman -S tesseract` /
> `sudo dnf install tesseract`.

**Display servers:** Both **Wayland** and **X11** are supported. Detection checks
`$WAYLAND_DISPLAY` first, then `$DISPLAY`. The capture path uses `slurp`+`grim`, which
work on either server. The hotkey backend is chosen automatically: Wayland uses an
evdev reader (see below); X11 uses an `x11rb` root-window grab.

## Quick start

1. Start the daemon (background; add to autostart if you like):

   ```bash
   ./target/release/pixelensd &
   ```

2. Grab a region:

   ```bash
   ./target/release/pixelens grab
   # alias: ./target/release/pixelens copy
   ```

   `slurp` opens; drag to select a rectangle, release to capture (Escape cancels).
   The daemon prints the saved screenshot path. With a running daemon the CLI exits
   non-zero if the daemon is down or capture fails, so it is safe in scripts.

3. (Optional) Check the daemon:

   ```bash
   ./target/release/pixelens status
   ```

## Global hotkey (UM1)

Bind a system-wide hotkey so you never type the command:

```bash
./target/release/pixelens hotkey enable
```

This writes a `pixelens-keyhook.service` systemd **user** unit and runs
`systemctl --user enable --now pixelens-keyhook`. The listener then starts
automatically on login.

- **Default combo:** `Super+Shift+T`.
- **Trigger:** press the combo anywhere; the listener connects to the daemon and fires
  a grab (same as `pixelens grab`).
- **Wayland:** the listener reads raw key events from `/dev/input/event*` via evdev,
  so **your user must be in the `input` group**:

  ```bash
  sudo usermod -aG input $USER
  # then log out and back in (group membership is read at login)
  ```

  Without the `input` group the listener logs `EvdevUnavailable` and the hotkey will
  not fire. Verify with `groups`.
- **X11:** the listener grabs the combo on the root window via `x11rb`; no special
  group is needed.
- **Manage it:**
  ```bash
  ./target/release/pixelens hotkey status   # shows service state + combo + daemon up/down
  ./target/release/pixelens hotkey disable  # stops + disables the service
  ```
- **Change the combo:** set `general.hotkey` in the config file, or the
  `PIXELENS_HOTKEY` environment variable (e.g. `Super+Shift+S`). It must be
  `Mod+Mod+Key` form, where modifiers are `Super`/`Shift`/`Ctrl`/`Alt` (case-insensitive)
  and the key is a single letter or digit (e.g. `Super+Shift+T`).

## Configuration

Config file: `~/.config/pixelens/config.toml` (created on first run with defaults).

```toml
[general]
autostart = false
theme = "system"
hotkey = "Super+Shift+T"

[capture]
show_preview = false
```

- `general.hotkey` — the hotkey combo used by the listener (see above). **Consumed.**
- `general.autostart`, `general.theme`, `capture.show_preview` — **parsed but not yet
  read by the daemon** in this build. Editing them has no effect yet; they exist so
  the config schema is stable for upcoming milestones.

> The `pixelens config` CLI subcommand is currently a **stub** — edit the TOML file
> directly with a text editor.

## Troubleshooting

**Daemon not running**
```
error: daemon is not running. Start it with: pixelensd
```
Start it: `./target/release/pixelensd &` (or check it is enabled as a service).
The CLI and the hotkey listener both talk to the daemon over
`$XDG_RUNTIME_DIR/pixelens.sock` (or `/tmp/pixelens-<uid>.sock` if `XDG_RUNTIME_DIR`
is unset).

**Missing `slurp` / `grim`**
`pixelens grab` returns an error naming the missing tool. Install it via your package
manager (table above) and restart the daemon.

**Hotkey not firing on Wayland**
The evdev reader needs `/dev/input/event*` access. Ensure your user is in the `input`
group (`sudo usermod -aG input $USER`, then re-login) and confirm with `groups`.
`pixelens hotkey status` reports the service state; if the listener logged
`EvdevUnavailable`, the hotkey cannot fire until the group membership is fixed.

**Capture returns a file path, not text**
Expected in the current build — OCR (M5) and clipboard copy (M7) are not yet
implemented. `pixelens grab` captures a screenshot of the region and reports its path.

## Architecture

```
   hotkey (pixelens-keyhook)  ──┐
                                ├─▶  daemon (pixelensd)
   CLI (pixelens grab)        ──┘        │
                                         ▼
                              slurp (select) → grim (capture) → screenshot file
                                         │
                            OCR (Tesseract)  ── NOT YET (M5)
                                         │
                            clipboard + notification ── NOT YET (M7)
```

The daemon (`pixelensd`) owns the capture pipeline and a Unix-socket IPC server. The
CLI (`pixelens`) and the hotkey listener (`pixelens-keyhook`) are thin clients: both
connect to the socket and send `Command::Grab`. The hotkey listener is intentionally
dumb — all capture state stays in the daemon.

## Not yet implemented (planned)

Marked NOT YET so there is no confusion with shipping behavior:

- OCR text extraction and clipboard copy (M5 / M7).
- `show_preview`, `autostart`, `theme` config effects.
- `pixelens config` CLI management (M8).
- Windows support, system tray, and HUD/actions popup (see
  `pixelens-plan/13-roadmap-upgrades.md`).

## License

MIT
