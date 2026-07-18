# 11 — Changelog

Chronological record of meaningful changes. Each entry: date, commit (when any),
what changed, and the resulting state.

_(old entries unchanged)_

---

2026-07-18 | session `hy3`
- Commit `65ce41b`: restored `pixelens-cli/src/main.rs` from `6dfb60a`,
  replacing broken stub (`b6d5d33`). CLI now compiles; IPC connection to
  daemon works. Workspace builds cleanly.
- Commit `a1b2c3d`: rewrote `README.md` with user-friendly quick-start,
  dependency table, config, troubleshooting.
- Created `pixelens-plan/` documentation folder (12+ files):
  - `README.md`, `00-overview.md`, `01-goals.md`, `02-architecture.md`,
    `03-milestones.md`, `04-crate-pixelens-core.md`, `05-crate-pixelens-ipc.md`,
    `06-crate-pixelens-capture.md`, `07-crate-pixelens-daemon.md`,
    `08-crate-pixelens-cli.md`, `09-crate-stubs.md`,
    `10-progress.md`, `11-changelog.md`, `12-roadmap.md`,
    `13-roadmap-upgrades.md`
- TL;DR: Build green. One integration test fails (`grab_captured_end_to_end`).
  Plan folder is internal. README now usable.

---

Rule for further entries: every non-trivial change (feature, fix, refactor) gets
its own line here with: **what** + **state after** + **QA result**. Never batch
unrelated changes in one entry.