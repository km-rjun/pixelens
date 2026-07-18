# 10 — Progress (READ FIRST EACH SESSION)

This file is the live status snapshot. Update it whenever you change code or
learn new facts. Keep the "build status" honest — a green build is the only
acceptable state to declare work "done".

_Last updated: 2026-07-18 (session: hy3 — UM1 hardening slice 1)_

## Build status: 🟢 GREEN
- `cargo build --workspace` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `cargo fmt --all -- --check` clean.

## Test status: 🟢 ALL GREEN (4/4 daemon integration tests)
- `cargo test -p pixelens-daemon --test integration` → 4 passed; 0 failed.
  - `grab_captured_end_to_end` now PASSES (was the only failure).
- Root cause of prior failure was a **test-stub bug**, not the daemon:
  the `grim` stub used `head -c 1024 /dev/urandom`, but the test installs
  an isolated `$PATH` containing only the slurp/grim stubs, so `head` was
  not found (exit 127) → 0-byte file → pipeline reported a zero-byte error.
  Fixed by rewriting the stub to emit 1024 NUL bytes via POSIX-sh builtins only.
- This was **not** an environmental (headless) failure — it reproduced with
  real stubs and the daemon pipeline was always correct.

## Git status
- Branch: `main` @ `5c89d69` (ahead of `e939894` by fix + UM1 commits)
- Recent commit:
  - `5c89d69` fix(daemon): make integration grim stub self-contained (no external head)
- Remote push attempted; permission denied (expected — local only)
- Recent commits:
  - `f2a3b4c` style: clippy warning fix (needless return)
  - `65ce41b` fix(cli): restore IPC-based grab/status client (this session)
  - `b6d5d33` ⚠️ BROKEN CLI commit (reverted)
  - `e939894` feat: complete integration tests + documentation
  - `6dfb60a` feat(cli): implement real grab/status/stop over IPC (reference)

## What is actually working
- M1 setup ✅ · M2 display detection ✅ · M3 v1-Wayland slurp+grim path ✅ ·
  M6 IPC protocol + daemon server ✅ · CLI client ✅ (fixed)

## What is NOT done (next in line)
1. M5 OCR (Tesseract warm init) — without it, "text copied" is impossible.
2. M7 clipboard + notify (core loop: selection → text on clipboard).
3. M4 X11 backend, M8 config, M9 tray, M10 packaging, M11 full testing/release.

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
Pick your next milestone (recommended: M5 OCR → clipboard), run its QA checklist,
then I execute code-safety checks (`cargo fmt --check` + `cargo clippy -- -D warnings`)
before any push.

## Habits to keep
- After EVERY change: commit small + descriptive.
- Before claiming a milestone: `cargo build` + `cargo test` green.
- QA explicit per milestone (see roadmap files).
- No deviation from PRD vision: hotkey → select → text copied in <2s.