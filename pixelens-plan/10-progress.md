# 10 — Progress (READ FIRST EACH SESSION)

This file is the live status snapshot. Update it whenever you change code or
learn new facts. Keep the "build status" honest — a green build is the only
acceptable state to declare work "done".

_Last updated: 2026-07-18 (session: hy3)_

## Build status: 🟢 GREEN
- `cargo build` passes after restoring CLI from `6dfb60a`.
- Commit `65ce41b` fixes the regression.

## Test status: 🔴 ONE FAILING INTEGRATION TEST
- `cargo test` fails on `pixelens-daemon/tests/integration/grab_captured_end_to_end`
- Pre-existing failure unrelated to CLI fix; grim stub invocation mismatch.
- Other integration tests pass.

## Git status
- Branch: `main` @ `65ce41b` (ahead of `e939894` by 1 fix commit + plan files)
- Remote push attempted; permission denied (expected — local only)
- Recent commits:
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

## Documentation added (this session)
- `pixelens-plan/` — 12+ files covering overview, goals, architecture, milestones,
  per-crate docs, progress, changelog, and roadmap (both v1 and upgrades).
- `README.md` — rewritten with setup, quick-start, config, troubleshooting.

## Immediate next action
Pick your next milestone (recommended: M5 OCR → clipboard), run its QA checklist,
then I execute code-safety checks (`cargo fmt --check` + `cargo clippy -- -D warnings`)
before any push.

## Habits to keep
- After EVERY change: commit small + descriptive.
- Before claiming a milestone: `cargo build` + `cargo test` green.
- QA explicit per milestone (see roadmap files).
- No deviation from PRD vision: hotkey → select → text copied in <2s.