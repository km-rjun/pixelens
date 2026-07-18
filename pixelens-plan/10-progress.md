# 10 — Progress (READ FIRST EACH SESSION)

This file is the live status snapshot. Update it whenever you change code or
learn new facts. Keep the "build status" honest — a green build is the only
acceptable state to declare work "done".

_Last updated: 2026-07-18 (session: hy3)_

## Build status: 🟢 GREEN
- `cargo build` passes after restoring CLI from `6dfb60a`.
- Commit `65ce41b` fixes the regression.
- Commit `f2a3b4c` fixes clippy warning.

## Test status: 🔴 ONE FAILING INTEGRATION TEST
- `cargo test` fails on `pixelens-daemon/tests/integration/grab_captured_end_to_end`
- Pre-existing failure unrelated to CLI fix; grim stub invocation mismatch.
- Other integration tests pass.

## Git status
- Branch: `main` @ `f2a3b4c` (ahead of `e939894` by 2 fix commits + plan files)
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