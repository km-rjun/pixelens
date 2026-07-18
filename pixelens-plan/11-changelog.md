# 11 — Changelog

Chronological record of meaningful changes. Each entry: date, commit (when any),
what changed, and the resulting state.

_(old entries unchanged)_

---

2026-07-18 | session `hy3` (continued)
- Added 3 detailed upgrade design docs: `14-upgrade-m1-hotkey.md`,
  `15-upgrade-m2-windows.md`, `16-upgrade-m4-grab-ui.md`.
- Extended `03-milestones.md` with UM1–UM8 upgrade milestone section.
- Started **UM1 implementation**: new `pixelens-keyhook` crate.
- Build/fmt/clippy green after each atomic step (see commit log).
- Commits `65ce41b`, `a1b2c3d`: CLI restoration + README rewrite (see prior session logs).
- Created `pixelens-plan/` documentation folder (15 files):
  - `README.md`, 00-09 slice docs, `10-progress.md`, `11-changelog.md`,
    `12-roadmap.md`, `13-roadmap-upgrades.md`
- TL;DR: Build green. One integration test fails (`grab_captured_end_to_end`).
  Plan folder is internal. Code-safety verified (`cargo fmt --check` + `clippy -D warnings` both pass).

---

Rule for further entries: every non-trivial change (feature, fix, refactor) gets
its own line here with: **what** + **state after** + **QA result**. Never batch
unrelated changes in one entry.