# 14 — Upgrade M1: Hotkey Daemon (detailed design)

**Depends on**: v1.0 core loop (`hotkey → select → text copied`).
**Status**: 📋 planned · **Owner**: odin · **Blocker for**: Upgrade M3 (systemd autostart builds on this).

---

## 1. Problem statement

Today `pixelens grab` only fires when a human types it into a shell. The PRD's
whole premise is *press a key → text lands on clipboard*. Without a global
hotkey the product is a CLI toy, not a utility. This milestone closes that gap
on Linux (Wayland + X11). Windows hotkey lives in Upgrade M2.

## 2. Constraints (from PRD — non-negotiable)

- No DBus in v1/v1.x for the hotkey path; the hotkey listener talks to the
  daemon **only over the existing Unix socket** (same as the CLI).
- No menus, no confirmations. A hotkey press must trigger an immediate `grab`.
- Single logical change per commit. Build + tests green before "done".
- Hotkey daemon must **not** block or slow the capture loop.

## 3. Architecture

```
┌─────────────────┐     hotkey press      ┌──────────────────┐
│ pixelens-keyhook│ ───────────────────►  │   pixelensd       │
│ (listener proc) │   (Unix socket: Grab) │  (existing daemon)│
└─────────────────┘                       └──────────────────┘
        │                                          │
        │ spawn on enable                          │ IPC already exists
        ▼                                          ▼
 systemd --user unit                    grab → slurp+grim → OCR → clipboard
```

Two processes, one socket:
- **`pixelens-keyhook`** — a tiny long-lived listener that watches for the
  configured key combo and, on press, connects to the daemon socket and sends
  `Command::Grab` (exactly like the CLI does). It does NOT do capture itself.
- **`pixelensd`** — unchanged; already serves `Grab`.

This keeps the keyhook dumb and lets the daemon own all capture/OCR state
(warm Tesseract, single pipeline). It also means a hotkey press and a CLI
`pixelens grab` are indistinguishable to the daemon — one code path.

## 4. New crate: `pixelens-keyhook`

Add to workspace `members` and `workspace.dependencies`:

```toml
# Cargo.toml (workspace)
members = [ ..., "pixelens-keyhook" ]
pixelens-keyhook = { path = "pixelens-keyhook" }
```

Crate layout:
```
pixelens-keyhook/
├── Cargo.toml
└── src/
    ├── main.rs          # entry: parse args, pick backend, run loop
    ├── lib.rs           # HotkeyListener trait + Error
    ├── backend.rs       # enum dispatch on DisplayServer
    ├── wayland.rs       # Wayland backend (see §6)
    └── x11.rs           # X11 backend (see §7)
```

`lib.rs`:
```rust
pub trait HotkeyListener {
    /// Block until the listener is stopped (Ctrl-C / `stop` command).
    fn run(self) -> anyhow::Result<()>;
}

pub fn build(display: DisplayServer, combo: &KeyCombo) -> anyhow::Result<Box<dyn HotkeyListener>>;
```

`KeyCombo` (stored in config, default `Super+Shift+S`):
```rust
pub struct KeyCombo { pub mods: Vec<Mod>, pub key: String }
```

## 5. CLI surface

Extend `pixelens-cli` with a `hotkey` subcommand:
```
pixelens hotkey enable    # install + start systemd --user unit, persist flag
pixelens hotkey disable   # stop + disable unit, clear flag
pixelens hotkey status    # show: enabled?, backend, bound combo, daemon up?
```

`enable`:
1. Write `~/.config/systemd/user/pixelens-keyhook.service` (template in §8).
2. `systemctl --user daemon-reload`
3. `systemctl --user enable --now pixelens-keyhook`
4. Persist `hotkey.enabled = true` into `config.toml` (Upgrade M3 owns full
   config, but we seed this one key now — see §9).

`disable`: reverse steps 1–3, set `hotkey.enabled = false`.

`status`: read config + query `systemctl --user is-active pixelens-keyhook`.

> Note: `pixelens hotkey enable` requires the keyhook binary to be on `PATH`
> (installed). For local dev, fall back to spawning the binary directly if
> systemd is unavailable (e.g. in a container).

## 6. Wayland backend

Wayland has **no stable global hotkey protocol** broadly shipped. Two options:

- **(a) `xdotool`/XTest under XWayland** — unreliable; many wlroots compositors
  disable XWayland keyboard grabs.
- **(b) evdev direct read** — open `/dev/input/event*` (requires `input`
  group or `cap_sys_rawio`), parse key events, detect combo. Robust, compositor
  agnostic, no protocol dependency.

**Decision: (b) evdev.** Add dependency `evdev = "0.12"`. The listener opens
each event device, matches `EV_KEY` events for the configured modifiers + key,
and fires `Grab` when all are down. This is the same approach used by `evsieve`
and `hawck`. It needs the user in the `input` group (documented in README +
QA step).

Caveat: evdev reads are exclusive if we grab the device; we must **NOT** grab
(exclude `EVIOCGRAB`) so the rest of the desktop keeps working. We only *observe*.

## 7. X11 backend

Use `xdotool` (already a documented runtime dep for capture? No — capture uses
`slurp`+`grim` which are Wayland-only). For X11 we add `xdotool` as a runtime
dependency and shell out:

```
xdotool behave <key> keydown --shell ...   # or a small XCB poll loop
```

Simpler and dependency-light: a small poll loop using `x11rb` (pure-Rust X11)
listening on the root window for `KeyPress` with the combo. Add `x11rb = "0.13"`.
This avoids a shell-out and is testable. **Decision: `x11rb` poll loop.**

## 8. systemd unit template

`~/.config/systemd/user/pixelens-keyhook.service`:
```ini
[Unit]
Description=Pixelens global hotkey listener
PartOf=pixelens.service
After=pixelens.service

[Service]
Type=simple
ExecStart=%h/.cargo/bin/pixelens-keyhook
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

> Path `%h/.cargo/bin/pixelens-keyhook` is the install location; for system
> installs use `/usr/bin/pixelens-keyhook`. The unit is **user** scope — no root.

## 9. Config additions (seed only; full config = Upgrade M3)

```toml
[hotkey]
enabled = false
combo = "Super+Shift+S"   # mods: Super, Shift, Ctrl, Alt
```

`pixelens-config` already has a `model.rs` stub; we add a `HotkeyConfig`
struct now but only read/write this one section.

## 10. Files touched

| File | Change |
|------|--------|
| `Cargo.toml` | add `pixelens-keyhook` member + dep |
| `pixelens-keyhook/` (new) | whole crate |
| `pixelens-cli/src/main.rs` | `hotkey` subcommand |
| `pixelens-config/src/model.rs` | `HotkeyConfig` struct |
| `README.md` | hotkey setup + `input` group note |

## 11. QA checklist (mandatory before push)

1. Wayland: `pixelens hotkey enable` → press combo → slurp overlay appears →
   select → text on clipboard.
2. X11 (VM): same flow works.
3. `pixelens hotkey status` reports `active` and correct backend.
4. Press combo 10× rapidly → no zombie `pixelens-keyhook` processes, no
   duplicate grabs.
5. `pixelens hotkey disable` → combo no longer triggers; unit `inactive`.
6. Capture loop timing unchanged (hotkey listener is passive).

## 12. Safety gate before GitHub push

- `cargo fmt --check` clean
- `cargo clippy -- -D warnings` clean
- `cargo test --workspace` green
- Review: unit file contains **no** `User=root`, no `CapabilityBoundingSet`
  beyond what evdev needs, no network.
- evdev open must fail gracefully (clear error, not panic) if not in `input`
  group.
