# 10 — Progress (READ FIRST EACH SESSION)

This file is the live status snapshot. Update it whenever you change code or
learn new facts. Keep the "build status" honest — a green build is the only
acceptable state to declare work "done".

_Last updated: 2026-07-19 (session: hy3 — UM3 autostart)_

## Build status: 🟢 GREEN (per-crate) · ⚠️ workspace link pressure (disk ~83-94%)
- `cargo build -p pixelens-cli` (and other per-crate builds) pass.
- `cargo clippy --workspace --all-targets -- -D warnings` clean (per-crate
  compiles; the workspace link step is flaky only under disk pressure).
- `cargo fmt --all -- --check` clean.
- ENVIRONMENT NOTE: root FS routinely 83-100% full. A full `cargo build
  --workspace` / `cargo test --workspace` cannot LINK (ENOSPC bus-errors on
  this VM). Mitigation that works: `cargo clean` first, then per-crate
  `cargo build -p <crate>` + per-crate `cargo test -p <crate>`. Never claim a
  full-workspace link passed when it couldn't link.

## Test status: 🟢 ALL GREEN (per-crate)
- `cargo test -p pixelens-config` → 3 passed; 0 failed
  (`load_defaults_when_missing`, `load_round_trips`, `get_and_set_dotted_keys`).
  Tests write to `std::env::temp_dir()` — never touch real `~/.config`.
- Functional CLI smoke test (in a throwaway `$HOME`): `config list`,
  `config get <key>`, `config set <key> <value>` (writes TOML, creates
  parent dir), and unknown-key error all behave correctly.
- Live preview/hotkey QA NOT exercised (headless, no display). Config
  consumption is observable via daemon startup logs (M8).
- `cargo test -p pixelens-ocr` → 5 passed; 0 failed (sanitize + error paths).
- `cargo test -p pixelens-notify` → 1 passed; 0 failed (notification
  message strings). NEW in M7.
- `cargo test -p pixelens-daemon --lib` → 1 passed; 0 failed
  (`clipboard::tests::copy_text_returns_no_backend_without_clipboard_tools`). NEW in M7.
- `cargo test -p pixelens-daemon --test integration` → 4 passed; 0 failed
  (incl. `grab_captured_end_to_end`, which now drives capture → OCR →
  clipboard/notify branch with no clipboard tool present → log-and-continue).
- `cargo test -p pixelens-ipc` → 4 passed; 0 failed.
- NOTE: live clipboard/notify QA NOT exercised headless (no display, no
  wl-copy/xclip/notify-send). Logic verified via unit + integration sim.
- NOTE: a full `cargo test --workspace` could not be linked in this session
  due to disk-full linker bus-errors; per-crate tests above WERE run green.
- Root cause of prior failure was a **test-stub bug**, not the daemon:
  the `grim` stub used `head -c 1024 /dev/urandom`, but the test installs
  an isolated `$PATH` containing only the slurp/grim stubs, so `head` was
  not found (exit 127) → 0-byte file → pipeline reported a zero-byte error.
  Fixed by rewriting the stub to emit 1024 NUL bytes via POSIX-sh builtins only.
- This was **not** an environmental (headless) failure — it reproduced with
  real stubs and the daemon pipeline was always correct.

## Git status
- Branch: `features/core-loop` (local main pushed here; remote `main` is an
  UNRELATED/different-crate-layout history — do NOT push to `main`).
- M8 commits this session:
  - `feat(config): add TOML load/save + get/set` (pixelens-config/src/io.rs)
  - `feat(cli): implement config list/get/set` (pixelens-cli/src/main.rs)
  - `feat(keyhook+daemon): consume general.hotkey + show_preview from config` (M8)
  - `test(config): round-trip loader` (pixelens-config/src/io.rs tests)
  - `docs(plan): record M8 configuration` (this changelog/progress/milestone updates)
- Remote push: NOT pushed this session (branch `features/core-loop` already
  exists; remote `main` diverged — do not force-push).

## What is actually working
- M1 setup ✅ · M2 display detection ✅ (Wayland + X11) · M3 v1 slurp+grim capture path ✅
  (works on both Wayland and X11; grim has X11 support) · M4 X11 capture **NOT done**
  (`pixelens-capture/src/x11.rs` is a stub → `PixelensError::NotImplemented("X11CaptureProvider (M4)")`,
  per 03-milestones.md M4 = ❌) · M5 OCR **partial** — `TesseractOcrEngine` implemented +
  validates + wired into `handle_grab`, `GrabResponsePayload.text` populated) ·
  M6 IPC protocol + daemon server ✅ · CLI client ✅ (fixed) ·
  M7 Clipboard + Notifications ✅ — `pixelens-daemon::clipboard::copy_text`
  shells out to wl-copy/xclip/xsel (display-server aware, degrades gracefully);
  `pixelens-notify::NotifySend` shells out to `notify-send` (non-fatal);
  `handle_grab` copies non-empty text + fires "✓ Text copied to clipboard",
  or "No text found in selection." when OCR text is empty. UM1 global hotkey ✅
  (Wayland evdev + X11 x11rb in `pixelens-keyhook/src/x11.rs`; systemd --user service).
  NOTE: the X11 x11rb code is the **UM1 hotkey listener**, not M4 capture.

## What is NOT done (next in line)
1. M5/M7 live QA — engine + wiring done; real screenshot→text→clipboard→
   notify not exercised headless (needs a display; tesseract 5.5.0 IS
   installed here, but no Wayland/X11 session + no wl-copy/xclip/notify-send).
- M8 Configuration ✅ — `pixelens-config` now loads/saves `~/.config/pixelens/config.toml`
  (defaults when absent/invalid). `pixelens config list|get|set` implemented
  (dotted keys, typed set, parent-dir creation, clear errors). Daemon `run()`
  loads config and LOGS `general.hotkey` (resolved env>config>default) + gates
  `capture.show_preview` log. `pixelens-keyhook` uses `general.hotkey` as its
  default combo (env still wins). CLI `keyhook` reports the config default.
4. Config keys `autostart` / `theme` are parsed; `autostart` IS now consumed
   (UM3, see below). `theme` still parsed but not read by the daemon (UI work).

> Honesty note (2026-07-18): an earlier progress entry correctly stated the README
> overclaimed "text copied to clipboard in <2s" and "tesseract required at startup" —
> both inaccurate for the current build; README was rewritten to match code.
> CORRECTION (2026-07-18, later same session): a prior sentence here claimed "M4 X11 is
> DONE (x11rb root-window grab in pixelens-keyhook/src/x11.rs)". That was WRONG. The
> x11rb code is the **UM1 hotkey listener**, not M4 capture. M4 Capture (X11) in
> `pixelens-capture/src/x11.rs` is **still a stub** (`NotImplemented("X11CaptureProvider (M4)")`),
> consistent with 03-milestones.md (M4 = ❌). See "What is actually working" above.

## Upgrade roadmap (post-v1.0)
- Upgrade M1: Hotkey daemon — binds global hotkey, systemd service backend
- Upgrade M2: Windows support — replaces Snipping Tool, same 2s flow
- Upgrade M3: Systemd + autostart integration
- Upgrade M4: Grab UI / Actions popup — minimal HUD, no menus
- Upgrade M5: Portal-native capture speedup
- Upgrade M6: Multi-display smart selection
- Upgrade M7: On-demand vs continuous mode
- Upgrade M8: Tesseract performance tuning

See `13-roadmap-upgrades.md` for details. Each upgrade requires QA checkpoint
and safety gate before GitHub push.

## Documentation added (this session)
- `pixelens-plan/` — 18 files: overview, goals, architecture, milestones,
  per-crate docs, progress, changelog, v1 + upgrade roadmaps, and three
  detailed upgrade design docs (`14-upgrade-m1-hotkey.md`,
  `15-upgrade-m2-windows.md`, `16-upgrade-m4-grab-ui.md`).
- `README.md` — rewritten with setup, quick-start, config, troubleshooting.

## Upgrade milestone tracking (post-v1.0)
- UM1 Hotkey daemon — 🚧 in progress: `pixelens-keyhook` crate done, CLI
  `hotkey` subcommand done, systemd unit generation done. Manual QA pending
  (needs real Wayland/X11 session — headless env blocks it).
- UM2 Windows support — 📋 planned, design doc written
- UM3 Systemd + autostart — ✅ DONE (2026-07-19): `pixelens-cli autostart
  enable|disable|status` manages `~/.config/autostart/pixelens.desktop` (XDG
  autostart spec; honors XDG_CONFIG_HOME, falls back to ~/.config). Pure
  helpers `write_autostart_desktop`/`remove_autostart_desktop` are unit-tested
  (round-trip + idempotent remove). `config set general.autostart true|false`
  now keeps the .desktop in sync (best-effort, non-fatal). Complements UM1's
  systemd --user service. Manual QA of actual desktop autostart pending
  (needs a real session — headless blocks it).
- Config keys `autostart` / `theme`: `autostart` IS now consumed (UM3). `theme`
  still parsed but not read by the daemon (UI work). `show_preview` +
  `general.hotkey` consumed in M8.

## Immediate next action
M8 (configuration) ✅ and UM3 (autostart) ✅ are DONE and committed to
`features/core-loop`. Core loop + config + autostart are code-complete and
verified (fmt/clippy/per-crate build/test green; full-workspace link blocked by
disk ENOSPC, per-crate used instead). Remaining upgrade work: **UM2 Windows**,
**UM4 Grab UI**, UM5–UM8. Before claiming the full core loop works live, run
OCR+clipboard+notify QA on a real Wayland/X11 display with wl-copy/xclip +
notify-send. _Last updated: 2026-07-19 (session: hy3 — UM3 autostart)._

## Habits to keep
- After EVERY change: commit small + descriptive.
- Before claiming a milestone: `cargo build` + `cargo test` green.
- QA explicit per milestone (see roadmap files).
- No deviation from PRD vision: hotkey → select → text copied in <2s.