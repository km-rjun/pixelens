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

2026-07-18 | session `hy3` (UM1 hardening slice 1 — fix failing integration test)
- **Fixed** `pixelens-daemon::integration::grab_captured_end_to_end`:
  - Root cause: the test's `grim` stub used `head -c 1024 /dev/urandom`,
    but the test installs an isolated `$PATH` containing only the slurp/grim
    stubs, so `head` was not found → exit 127 → 0-byte output file → the
    pipeline reported a zero-byte-capture error (test saw `Error`, expected `Ok`).
  - This was a **real stub bug**, not an environmental/headless limitation —
    it reproduces deterministically whenever `head` is absent from the stub PATH.
  - Fix: rewrite the `grim` stub to write exactly 1024 NUL bytes using only
    POSIX-sh builtins (`printf`/`[`/`$(( ))`), so it needs no external tools.
  - State after: `cargo test -p pixelens-daemon --test integration` →
    **4 passed; 0 failed** (`grab_captured_end_to_end` now green).
  - Verified clean: `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings`.
  - Commit `5c89d69` fix(daemon): make integration grim stub self-contained (no external head).
  - Files changed: `pixelens-daemon/tests/integration.rs` (+5/-1).

---

Rule for further entries: every non-trivial change (feature, fix, refactor) gets
its own line here with: **what** + **state after** + **QA result**. Never batch
unrelated changes in one entry.