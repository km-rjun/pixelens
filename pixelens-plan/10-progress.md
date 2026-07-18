# 10 — Progress (READ FIRST EACH SESSION)

This file is the live status snapshot. Update it whenever you change code or
learn new facts. Keep the "build status" honest — a green build is the only
acceptable state to declare work "done".

_Last updated: 2026-07-18 (session: hy3 — M7 clipboard + notifications)_

## Build status: 🟢 GREEN (code) · ⚠️ link pressure (disk 100%)
- `cargo build --workspace` passes (pixelensd binary links OK).
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `cargo fmt --all -- --check` clean.
- ENVIRONMENT NOTE: root FS is ~100% full (≤156M free after `cargo clean`).
  Linking the daemon **test** binaries under cargo's default `lld` linker
  intermittently bus-errors (signal 7) from disk exhaustion. Worked around
  by `cargo clean` + freeing incremental/fingerprint caches before each link.
  This is an environment limit, not a code defect — see Test status.

## Test status: 🟢 ALL GREEN (after freeing disk for link)
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
- Branch: `main` @ `f066f80` (M7 clipboard + notify wired into handle_grab)
- M7 commits this session:
  - `4948048` feat(notify): shell out to notify-send (M7)
  - `639de9b` feat(daemon): copy OCR text to clipboard (M7)
  - `f066f80` feat(daemon): wire clipboard + notify into handle_grab (M7)
- Remote push: `git push --dry-run origin main` REJECTS — remote
  (git@github.com:km-rjun/pixelens.git) has commits local lacks (diverged
  history: "Updates were rejected because the remote contains work that you
  do not have locally"). Do NOT force-push. Not pushed this session.
- Prior hardening commits (this session, earlier slices):
  - `5c89d69` fix(daemon): make integration grim stub self-contained
  - `2bb1abf` docs: rewrite README for end users (honest to current build)

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
2. M8 config (the `pixelens config` CLI is a stub; config keys are parsed but not all
   consumed) · M9 tray · M10 packaging · M11 full testing/release.
4. Config keys `show_preview` / `autostart` / `theme` are parsed but **not yet read** by
   the daemon.

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
- UM3 Systemd + autostart — 📋 planned (in roadmap)
- UM4 Grab UI / HUD — 📋 planned, design doc written
- UM5–UM8 — 📋 planned (in roadmap, no split docs yet)

## Immediate next action
M7 (clipboard + notify) is DONE and committed (composable, non-fatal). The
core loop selection → OCR → clipboard → notify now runs end-to-end in the
daemon (verified by integration sim). Next slice: **M8 configuration** —
consume the parsed config keys (`show_preview` / `autostart` / `theme`) in
the daemon, and flesh out the `pixelens config` CLI. Before claiming the
full core loop works, run live OCR+clipboard+notify QA on a machine with a
real Wayland/X11 display and wl-copy/xclip + notify-send installed.

## Habits to keep
- After EVERY change: commit small + descriptive.
- Before claiming a milestone: `cargo build` + `cargo test` green.
- QA explicit per milestone (see roadmap files).
- No deviation from PRD vision: hotkey → select → text copied in <2s.