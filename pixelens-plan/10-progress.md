# 10 — Progress (READ FIRST EACH SESSION)

This file is the live status snapshot. Update it whenever you change code or
learn new facts. Keep the "build status" honest — a green build is the only
acceptable state to declare work "done".

_Last updated: 2026-07-18 (session: hy3)_

## Build status: 🔴 RED
`cargo build` FAILS because `pixelens-cli/src/main.rs` (commit `b6d5d33`) is a
broken hand-rolled stub. See `08-crate-pixelens-cli.md`. Everything else in the
workspace compiles.

## Git status
- Branch: `main` @ `b6d5d33` (ahead of `e939894` by the bad CLI commit).
- Remote `origin` is GitHub `km-rjun/pixelens`. The bad commit was pushed.
- Commits of record (newest first):
  - `b6d5d33` ⚠️ Implement pixelens grab workflow with error handling  ← BROKE CLI
  - `e939894` feat: complete integration tests + documentation
  - `6dfb60a` feat(cli): implement real grab/status/stop over IPC        ← good CLI
  - `1c1a929` feat(daemon): add IPC server and command dispatcher
  - `cb42edc` feat(ipc): add typed Grab response payload and helper constructors
  - `9717349` feat(capture): add slurp+grim capture pipeline orchestrator
  - `5ce1842` feat(capture): add which() helper for $PATH tool lookup
  - `cb8ca85` feat(capture): introduce slurp+grim v1 path with typed capture error
  - `ff60266` chore: commit cargo.lock for binary workspace
  - `7aa9887` docs: add project readme
  - `9422191` ci: add github actions workflow for check, build, and test
  - `b3175cf` chore: add tests directory for cross-crate integration tests
  - `2f5fed7` docs: add architecture and milestone tracking notes
  - `ef3d56f` feat(cli): add pixelens binary with command parser
  - `c8e54e1` feat(daemon): add pixelensd binary stub

## What is actually working
- M1 setup ✅ · M2 display detection ✅ · M3 v1-Wayland slurp+grim path ✅ ·
  M6 IPC protocol + daemon server ✅ (CLI client side ⚠️ regressed).

## What is NOT done (next in line)
1. ⚠️ **FIX CLI** (`08`) — restore IPC client, get `cargo build` green.
2. M7 clipboard + notify (so the captured image actually becomes text → clipboard).
3. M5 OCR (Tesseract warm init) — without it, "text copied" is impossible.
4. M4 X11 backend, M8 config, M9 tray, M10 packaging, M11 full testing/release.

## Immediate next action
Restore `pixelens-cli/src/main.rs` from `6dfb60a`, verify `cargo build`,
commit as a fix. Then proceed to OCR+clipboard (the path to the actual PRD
success criterion).

## Habits to keep
- After EVERY change: commit small + descriptive (PRD §Git Workflow).
- Before claiming a milestone: `cargo build` + `cargo test` green.
- Don't trust prior "done/pushed" claims — verify against the tree.
