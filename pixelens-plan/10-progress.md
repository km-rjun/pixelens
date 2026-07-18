# 10 — Progress (READ FIRST EACH SESSION)

This file is the live status snapshot. Update it whenever you change code or
learn new facts. Keep the "build status" honest — a green build is the only
acceptable state to declare work "done".

_Last updated: 2026-07-18 (session: hy3 — M5 OCR engine + daemon wiring)_

## Build status: 🟢 GREEN
- `cargo build --workspace` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `cargo fmt --all -- --check` clean.

## Test status: 🟢 ALL GREEN
- `cargo test -p pixelens-ocr` → 5 passed; 0 failed (sanitize + error paths).
- `cargo test -p pixelens-daemon` → 4 passed; 0 failed (incl. `grab_captured_end_to_end`,
  `grab_payload_round_trips` now exercises the new `text` field).
- `cargo test -p pixelens-ipc` → 4 passed; 0 failed.
- NOTE: live OCR end-to-end (real screenshot → tesseract → text) NOT exercised headless.
- Root cause of prior failure was a **test-stub bug**, not the daemon:
  the `grim` stub used `head -c 1024 /dev/urandom`, but the test installs
  an isolated `$PATH` containing only the slurp/grim stubs, so `head` was
  not found (exit 127) → 0-byte file → pipeline reported a zero-byte error.
  Fixed by rewriting the stub to emit 1024 NUL bytes via POSIX-sh builtins only.
- This was **not** an environmental (headless) failure — it reproduced with
  real stubs and the daemon pipeline was always correct.

## Git status
- Branch: `main` @ `d22ecab` (M5 OCR temp-file .bmp fix)
- Recent M5 commits:
  - `d22ecab` fix(ocr): name temp image .bmp to match BMP-encoded bytes
  - `2e4355e` feat(ocr): implement TesseractOcrEngine extract_text + validation
  - `c45e3b2` feat(ipc): add backward-compatible text field to GrabResponsePayload
  - `766a034` feat(daemon): warm-init OCR engine + run on grab (M5)
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
  validated + wired into `handle_grab`, `GrabResponsePayload.text` populated (OCR result still
  NOT copied to clipboard — that's M7) · M6 IPC protocol + daemon server ✅ · CLI client ✅ (fixed) ·
  UM1 global hotkey ✅ (Wayland evdev + X11 x11rb in `pixelens-keyhook/src/x11.rs`;
  systemd --user service). NOTE: the X11 x11rb code is the **UM1 hotkey listener**, not M4 capture.

## What is NOT done (next in line)
1. M5 OCR live QA — engine + wiring done; real screenshot→text not exercised headless
   (needs a display; tesseract 5.5.0 IS installed here).
2. M7 clipboard + notify (core loop: selection → OCR text → clipboard). M5 already
   populates `GrabResponsePayload.text`; M7 copies it to the clipboard.
3. M8 config (the `pixelens config` CLI is a stub; config keys are parsed but not all
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
M5 engine + wiring is DONE (committed). Next slice: **M7 clipboard** — copy the
`GrabResponsePayload.text` to the system clipboard (and notify). Live OCR QA
(real screenshot→text) should be run on a machine with a display before claiming
the full core loop works.

## Habits to keep
- After EVERY change: commit small + descriptive.
- Before claiming a milestone: `cargo build` + `cargo test` green.
- QA explicit per milestone (see roadmap files).
- No deviation from PRD vision: hotkey → select → text copied in <2s.