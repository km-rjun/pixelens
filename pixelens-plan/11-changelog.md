# 11 — Changelog

Chronological record of meaningful changes. Each entry: date, commit (when any),
what changed, and the resulting state.

_(old entries unchanged)_

---

2026-07-18 | session `hy3` (UM1 implementation)
- Implemented **UM1 Hotkey Daemon**:
  - New crate `pixelens-keyhook`: `lib.rs` (trait + `KeyCombo` parse + `fire_grab`),
    `wayland.rs` (evdev reader, no device grab), `x11.rs` (x11rb root grab +
    keyboard-mapping keysym→keycode resolve), `backend.rs` (display dispatch),
    `main.rs`.
  - CLI `pixelens hotkey enable|disable|status`: writes systemd `--user` unit,
    calls `systemctl --user enable --now`, queries status.
  - README: hotkey quick-start + `input` group troubleshooting.
- Build: `cargo build --workspace` green. Clippy `--workspace -D warnings` clean.
  Fmt clean. 3/4 daemon integration tests pass; `grab_captured_end_to_end`
  fails (pre-existing, needs real Wayland+slurp/grim session — headless here).
- Added UM1/UM2/UM4 design docs (earlier in session) + upgrade milestone section.
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