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

---

2026-07-18 | session `hy3` (UM1 hardening slice 2 — user README rewrite)
- **Rewrote `README.md`** to be honest to the *current* code, not the v1 vision:
  - New "What works today" table: capture (slurp+grim) ✅, daemon+CLI ✅, hotkey ✅;
    OCR ❌ NOT YET (M5), clipboard ❌ NOT YET (M7), `pixelens config` CLI ❌ stub,
    `show_preview`/`autostart`/`theme` parsed-but-not-consumed.
  - Corrected prior overclaims: "text copied to clipboard in <2s" and "tesseract
    required at daemon startup" are inaccurate for this build. `handle_grab` returns
    a screenshot **file path**, not OCR text. Tesseract is noted as a *future* dep.
  - Documented required runtime deps (slurp, grim) with per-distro install commands;
    marked tesseract as not-yet-consumed.
  - Documented X11 as **supported** (x11rb), correcting the "X11 stubbed" assumption.
  - Documented hotkey: `pixelens hotkey enable|disable|status`, default `Super+Shift+T`,
    Wayland `input`-group requirement, combo config via `general.hotkey` / `PIXELENS_HOTKEY`.
  - Config file path, real `[general]`/`[capture]` keys, and a note that the `config`
    CLI is a stub (edit TOML directly).
  - Troubleshooting: daemon down, missing slurp/grim, Wayland hotkey not firing
    (input group), and "capture returns a path not text" expectation.
- Also corrected `10-progress.md`: M5/M7 still pending; M4 X11 capture still a stub (see correction entry below).
- QA: `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets
  -- -D warnings` clean (no Rust changes, README only). No code behavior changed.
- Commit `2bb1abf` docs: rewrite README for end users (honest to current build).
  Files: `README.md` (+157/-76).

---

2026-07-18 | session `hy3` (REVISE — correct M4 X11 capture status)
- **CORRECTION** of an earlier mislabel: `10-progress.md` (and a prior 11-changelog
  line under the README-rewrite slice) claimed "M4 X11 is DONE (x11rb root-window
  grab in pixelens-keyhook/src/x11.rs)". This was WRONG.
  - The x11rb code in `pixelens-keyhook/src/x11.rs` is the **UM1 global hotkey
    listener**, not M4 capture.
  - **M4 Capture (X11)** lives in `pixelens-capture/src/x11.rs` and is **still a
    stub**: `X11CaptureProvider` returns `PixelensError::NotImplemented("X11CaptureProvider (M4)")`.
  - This matches `03-milestones.md` (## M4 — Capture (X11) — ❌, stub only).
- Fixed `10-progress.md` "What is actually working" + Honesty note, and the inline
  changelog line, to reflect: M4 = NOT done (stub); x11rb X11 belongs to UM1 hotkey.
- State after: no Rust changed; plan files consistent with 03-milestones.md.
- QA: `cargo fmt --all -- --check` clean (no Rust changes).

---

2026-07-18 | session `hy3` (M5 — OCR engine + daemon wiring)
- **Implemented `TesseractOcrEngine`** in `pixelens-ocr/src/lib.rs` (was a skeleton):
  - `new()` validates tesseract presence via `tesseract --version`; returns
    `OcrError::EngineMissing` if absent (PRD dependency validation at startup).
  - `with_config(lang, psm)` + `extract_from_path(&Path)`: shells out to
    `tesseract <in> <out_base> -l <lang> --psm <n>`. NOTE: it does NOT use
    stdout — tesseract treats `<out_base>` as a *basename* and writes
    `<out_base>.txt`; the engine reads that `.txt` file and returns the
    trimmed text. Handles non-zero exit, missing `.txt` output, empty
    output, unsupported-image (wrong format) errors.
  - `extract_text(&CaptureImage)` trait impl: encodes `CaptureImage` to a 24-bit
    BMP (hand-rolled, no extra crate deps) and writes it to a temp file with a
    `.bmp` extension, then reuses `extract_from_path`.
    BMP chosen over PNG/PPM to avoid png/image/flate2 deps and guarantee
    leptonica support; the temp file uses a `.bmp` extension to match its
    real content (fixed 2026-07-18: it previously used a misleading `.png`
    extension).
  - Pure `sanitize_text()` helper (trim lines, collapse blank runs, drop trailing
    blank lines) so the post-processing logic is unit-testable without tesseract.
- **IPC** (`pixelens-ipc/src/protocol.rs`): added `pub text: String` to
  `GrabResponsePayload` with `#[serde(default)]` (backward-compatible).
- **Daemon wiring** (`pixelens-daemon`):
  - `DaemonState` now holds `ocr: Option<TesseractOcrEngine>` (warm).
  - `run()` warm-inits the engine after the pipeline is ready; missing tesseract
    logs a warn and continues (capture still works, grabs return empty text).
  - `handle_grab` runs `extract_from_path` on the captured PNG after a
    `GrabOutcome::Captured` and attaches `text` to the response. OCR failure is
    non-fatal: capture still returns (path + empty text). **M7 clipboard copy
    is deliberately NOT done here** — next iteration.
- **Tests** (`cargo test -p pixelens-ocr`): 5 passed. Covers `sanitize_text`
  behavior + the error path (`new()` NotFound branch, missing-file `extract_from_path`
  error) WITHOUT requiring a tesseract binary at test time.
- QA: `cargo build --workspace` green; `cargo clippy --workspace --all-targets
  -- -D warnings` clean; `cargo fmt --all -- --check` clean.
  `cargo test -p pixelens-daemon` 4/4 pass (incl. `grab_payload_round_trips`);
  `cargo test -p pixelens-ipc` 4/4 pass.
- **Live OCR end-to-end (real screenshot → text) NOT exercised**: headless VM has
  no display, so a real Wayland/X11 capture cannot be produced. Tesseract 5.5.0 is
  installed on this host and the engine *would* run, but only on a real image.
- Commits: `2e4355e` (ocr engine), `c45e3b2` (ipc text field), `766a034` (daemon wiring).
- GitHub: NOT pushed — `git push --dry-run origin main` REJECTS (remote diverged).
  Do not force-push.

---

2026-07-18 | session `hy3` (REVISE — fix OCR temp-file extension + doc accuracy)
- **Code fix** in `pixelens-ocr/src/lib.rs`: `extract_text` encoded `CaptureImage`
  to 24-bit BMP bytes but wrote the temp file with a `.png` extension
  (misleading to readers/tesseract). Renamed the temp extension to `.bmp`
  and renamed the helper `encode_image_as_png` → `encode_image_as_bmp` to match.
  Behavior unchanged: tesseract still reads the file via `extract_from_path`
  (which reads `<out_base>.txt`), NOT stdout.
- **Doc accuracy correction**: prior M5 changelog text claimed
  `extract_from_path` "shells out to `tesseract <in> stdout`". That was WRONG.
  The engine writes a temp file and reads `<basename>.txt`; it does not use
  stdout. Corrected here and in the M5 entry above.
- Also corrected: 10-progress.md + 03-milestones.md now state the temp image
  extension is `.bmp`.
- QA: `cargo fmt --all` clean; `cargo clippy --workspace --all-targets -- -D warnings`
  clean; `cargo test -p pixelens-ocr` 5/5 pass.
- Commit `d22ecab` fix(ocr): name temp image .bmp to match BMP-encoded bytes.
- GitHub: NOT pushed — `git push --dry-run origin main` still REJECTS (remote
  has commits local lacks; diverged history). Do not force-push.

---

Rule for further entries: every non-trivial change (feature, fix, refactor) gets
its own line here with: **what** + **state after** + **QA result**. Never batch
unrelated changes in one entry.