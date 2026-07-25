# Pixelens

Pixelens is a keyboard-first screen-text utility for **Linux** and **Windows**.
Press a hotkey (or run a command), select a screen region, and the extracted
**text is copied to your clipboard** — no menus, no cloud, no AI.
Grab-to-clipboard in under two seconds.

Pixelens stays a *utility*: a fast hotkey-driven capture pipeline, not a GUI
app. There is no tray, no main window, and no point-and-click interface. (A
visual HUD overlay was considered and deliberately deferred — see
[Status](#status--roadmap).)

---

## Install

### Prerequisites

- **Rust** (edition 2021) to build from source.
- A **Wayland or X11** session.
- Runtime capture/OCR tools on `$PATH` (see table).

| Tool | Purpose | Debian/Ubuntu | Arch | Fedora |
|------|---------|---------------|------|--------|
| `slurp` | Region selection | `sudo apt install slurp` | `sudo pacman -S slurp` | `sudo dnf install slurp` |
| `grim` | Screen capture | `sudo apt install grim` | `sudo pacman -S grim` | `sudo dnf install grim` |
| `tesseract` | OCR (text extraction) | `sudo apt install tesseract-ocr` | `sudo pacman -S tesseract` | `sudo dnf install tesseract` |
| `wl-copy` / `xclip` | Clipboard write | `sudo apt install wl-clipboard` / `sudo apt install xclip` | `sudo pacman -S wl-clipboard` / `xclip` | `sudo dnf install wl-clipboard` / `xclip` |
| `notify-send` | Desktop notification | `sudo apt install libnotify-bin` | `sudo pacman -S libnotify` | `sudo dnf install libnotify` |

> Clipboard + notify tools are auto-detected: Wayland uses `wl-copy`, X11 uses
> `xclip`; notifications use `notify-send`. If a tool is missing, that step is
> skipped with a log line rather than failing the grab.

### Build from source

```bash
git clone https://github.com/km-rjun/pixelens
cd pixelens
cargo build --release
```

This produces three binaries in `target/release/`:

- **`pixelensd`** — the background **daemon** (owns capture + OCR + clipboard + IPC).
- **`pixelens`** — the **CLI** client (talks to the daemon over a Unix socket).
- **`pixelens-keyhook`** — the **global hotkey listener** (spawned by the daemon/cli).

### First run

```bash
./target/release/pixelensd &          # start the daemon (background)
./target/release/pixelens status      # confirm it is up
```

The daemon writes a default config to `~/.config/pixelens/config.toml` on first
run if one does not exist.

### Windows

Pixelens on Windows ("UM2") uses the same architecture as Linux: a daemon, a
CLI client, and a global hotkey listener — but the capture path is the WinRT
**Graphics Capture Picker** (the same machinery the Snipping Tool / `Win+Shift+S`
shell experience uses), which replaces `slurp`/`grim`. Clipboard and
notifications use `arboard` / WinRT respectively.

**Build from source** (requires the `x86_64-pc-windows-msvc` target and the
Windows SDK):

```powershell
rustup target add x86_64-pc-windows-msvc
git clone https://github.com/km-rjun/pixelens
cd pixelens
cargo build --release --target x86_64-pc-windows-msvc
```

This produces `target/x86_64-pc-windows-msvc/release/pixelensd.exe`,
`pixelens.exe`, and `pixelens-keyhook.exe`.

**Windows specifics:**

- **Hotkey:** `Win+Shift+S` (the native Snip shortcut). Press it, pick a region
  in the system capture picker, and the text is OCR'd and copied to the
  clipboard with a toast.
- **No `slurp`/`grim`/`tesseract`/`wl-copy`/`notify-send`** — all of those are
  Linux-only. The Windows build links `tesseract` via the native path and uses
  platform clipboard/notification APIs.
- **IPC transport** is a named pipe (`\\.\pipe\pixelens`) instead of a Unix
  socket.
- **All tests pass** on Windows (`cargo test --target x86_64-pc-windows-msvc --workspace`).

---

## How to use

### The fast path (recommended)

1. Enable the global hotkey so you never type a command:

   ```bash
   ./target/release/pixelens hotkey enable
   ```

   This installs a systemd **user** service (`pixelens-keyhook`) that starts on
   login. Default combo: **`Super+Shift+T`**.

2. Press the hotkey anywhere. `slurp` opens — drag to select a rectangle and
   release. Escape cancels.

3. The selected region is OCR'd and the **text is copied to your clipboard**,
   with a desktop notification confirming the grab.

That is the whole loop. No windows, no confirmations.

### CLI

```bash
pixelens grab            # capture a region now (alias: pixelens copy)
pixelens status          # daemon health + last-grab summary
pixelens cancel          # cancel an in-progress grab
pixelens hotkey enable   # install + start the hotkey systemd user service
pixelens hotkey disable  # stop + remove it
pixelens hotkey status   # service state, combo, daemon up/down
pixelens autostart enable   # start Pixelens automatically on login (UM3)
pixelens autostart disable  # remove the autostart
pixelens autostart status   # show autostart state
pixelens config list    # print the resolved config
pixelens config get <key>
pixelens config set <key> <value>
```

Examples:

```bash
pixelens config set general.hotkey Super+Shift+S
pixelens config set capture.show_preview true
pixelens config get gui.hud_enabled
```

### Hotkey on Wayland vs X11

- **Wayland:** the listener reads raw key events from `/dev/input/event*` via
  evdev, so **your user must be in the `input` group**:

  ```bash
  sudo usermod -aG input $USER
  # log out and back in for the group to take effect
  ```

  Without it the listener logs `EvdevUnavailable` and the hotkey will not fire
  (`groups` to verify).

- **X11:** the combo is grabbed on the root window via `x11rb`; no special
  group needed.

### Configuration

File: `~/.config/pixelens/config.toml`

```toml
[general]
autostart = false
theme = "system"            # parsed, not yet used
hotkey = "Super+Shift+T"

[capture]
show_preview = false       # open the screenshot file after grab

[ocr]
engine = "tesseract"

[gui]
hud_enabled = true         # master switch for the (deferred) HUD
hud_timeout_ms = 1500
```

| Key | Effect | State |
|-----|--------|-------|
| `general.hotkey` | Hotkey combo | **Consumed** |
| `general.autostart` | Auto-start on login | **Consumed** (UM3) |
| `capture.show_preview` | Open screenshot after grab | **Consumed** |
| `ocr.engine` | OCR engine selector | **Consumed** (tesseract) |
| `general.theme` | UI theme | Parsed, not yet used |
| `gui.hud_enabled` / `gui.hud_timeout_ms` | HUD flags | Parsed; HUD deferred |

Change the combo via the file or the `PIXELENS_HOTKEY` env var (form
`Mod+Mod+Key`, e.g. `Super+Shift+S`; modifiers `Super`/`Shift`/`Ctrl`/`Alt`,
key is a single letter/digit).

---

## Status & roadmap

**Shipped and working:**

- Region capture (`slurp` + `grim`) on Wayland **and** X11.
- Background daemon + CLI over a Unix socket (Linux) / named pipe (Windows).
- Global hotkey via systemd user service (UM1), auto-start on login (UM3).
- OCR via Tesseract → text copied to clipboard + desktop notification (M5/M7).
- `pixelens config` CLI: `list` / `get` / `set` (M8).
- Grab-backend IPC for one-shot preview override + display re-detect (UM4
  backend).
- **Windows support (UM2):** the full `#[cfg(windows)]` pipeline is wired
  (WinRT capture picker, `RegisterHotKey` listener, named-pipe IPC,
  `arboard`/WinRT clipboard + notifications) and type-checks against
  `x86_64-pc-windows-msvc`. A native Windows run to confirm the picker/capture
  loop end-to-end is still pending.

**Deliberately deferred (not bugs — scope decisions):**

- **Visual HUD / actions popup (`pixelens-gui`):** scrapped for now. Pixelens
  is a utility, not a GUI app. Only minimal, essential grab affordances will be
  considered later if needed — and the same applies to the Windows release.
- **History / recall of past grabs:** coming later, not in this build.
- **`general.theme`:** parsed but unused until a UI exists.

---

## Troubleshooting

**Daemon not running**
```
error: daemon is not running. Start it with: pixelensd
```
Start it: `./target/release/pixelensd &` (or enable autostart). The CLI and
hotkey listener both talk to the daemon over `$XDG_RUNTIME_DIR/pixelens.sock`
(or `/tmp/pixelens-<uid>.sock` if `XDG_RUNTIME_DIR` is unset).

**Missing `slurp` / `grim` / `tesseract`**
`pixelens grab` returns an error naming the missing tool. Install it (table
above) and retry — no daemon restart needed for tool presence.

**Hotkey not firing on Wayland**
The evdev reader needs `/dev/input/event*` access. Ensure your user is in the
`input` group (`sudo usermod -aG input $USER`, then re-login) and confirm with
`groups`. `pixelens hotkey status` reports the service state; if the listener
logged `EvdevUnavailable`, fix the group membership.

**Grab succeeds but clipboard is empty**
Clipboard write falls back across `wl-copy` (Wayland) / `xclip` (X11). If both
are absent, the text is still reported via notification and `pixelens status`,
but not placed on the clipboard. Install the matching tool for your session.

---

## How it works (technical)

```
   hotkey (pixelens-keyhook)  ──┐
                                 ├─▶  daemon (pixelensd)
   CLI (pixelens grab)        ──┘        │
                                          ▼
                            slurp (select) → grim (capture) → screenshot
                                          │
                            OCR (Tesseract) → extracted text
                                          │
                            clipboard (wl-copy / xclip) + notify-send
```

- **`pixelensd`** owns the capture pipeline and a Unix-socket IPC server. It
  detects the display server (`$WAYLAND_DISPLAY`, else `$DISPLAY`), runs
  `slurp` for selection and `grim` for capture, OCRs the result with
  Tesseract, and copies the text to the clipboard.
- **`pixelens`** (CLI) and **`pixelens-keyhook`** (hotkey listener) are thin
  clients: both connect to the socket and send a `Command`. The listener is
  intentionally dumb — all capture state lives in the daemon.
- **One-shot overrides (UM4 backend):** `setpreview` / `redetect` IPC commands
  let a future HUD tune a single grab (preview on/off, re-run display
  detection) without editing config. The default path is unchanged when no
  override is set.

## License

MIT
