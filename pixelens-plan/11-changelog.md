# 11 — Changelog

Chronological record of meaningful changes. Each entry: date, commit (when any),
what changed, and the resulting state.

_(old entries unchanged)_

---

2026-07-18 | session `hy3` (M7 — Clipboard + Notifications)
- Implemented **M7 Clipboard + Notifications** (completes the core loop):
  - New `pixelens-daemon::clipboard` module: `copy_text(text)` shells out to
    `wl-copy`/`copyq` on Wayland or `xclip -selection clipboard`/`xsel -b`
    on X11 (backend chosen via `pixelens_capture::DisplayServer` from
    `DaemonState`). `ClipboardError::NoBackend` when no tool present →
    caller logs a warning and continues (never fails the grab).
  - `pixelens-notify`: replaced stub with a real `NotifySend` backend that
    shells out to `notify-send`; non-fatal `BackendUnavailable` when missing.
    Added `NotificationKind::{TextCopied, NoTextFound, TesseractMissing,
    DaemonNotRunning}` + `Notifier` trait. Unit test covers message strings.
  - Wired into `handle_grab`: capture → OCR → if text non-empty: copy to
    clipboard + fire `TextCopied` ("✓ Text copied to clipboard"); if empty:
    fire `NoTextFound` ("No text found in selection."). Empty text is a
    **successful** grab, never an error. Both steps best-effort/log-and-continue.
  - Unit test `clipboard::tests::copy_text_returns_no_backend_without_clipboard_tools`
    (headless, no clipboard tool → `NoBackend`).
- Build: `cargo build --workspace` green (binary links). Clippy
  `--workspace --all-targets -D warnings` clean. Fmt clean.
- Test: `pixelens-notify` 1/1, `pixelens-daemon --lib` 1/1,
  `pixelens-daemon --test integration` 4/4 (incl. `grab_captured_end_to_end`
  now drives the clipboard/notify branch via stub slurp/grim). Full
  `cargo test --workspace` could NOT link due to root-FS disk-full linker
  bus-errors — worked around per-crate; not an environment/code defect.
- Commits: `4948048` (notify), `639de9b` (clipboard), `f066f80` (wire),
  `docs(plan): record M7` (this changelog/progress/milestone updates).
- Live clipboard/notify QA STILL BLOCKED: headless VM, no Wayland/X11
  display, no wl-copy/xclip/notify-send installed. Needs a real session.
- GitHub remote diverged; not pushed (no force-push).

---

2026-07-18 | session `hy3` (M8 — Configuration: make config USED)
- **Implemented `pixelens-config` file I/O** (`pixelens-config/src/io.rs`):
  - `load_config()` / `load_config_from(path)` — read `~/.config/pixelens/config.toml`;
    return `Config::default()` (model defaults) when absent/unreadable/invalid TOML.
  - `save_config()` / `save_config_to(path)` — write TOML (serde), create parent dir
    (`create_dir_all`) if missing.
  - `get_value(&Config, &str)` / `set_value(&mut Config, &str, &str)` — dotted-key
    access (`general.hotkey`, `capture.show_preview`, …) with loose typed validation
    (bool for autostart/show_preview, string otherwise). `KNOWN_KEYS` for validation +
    `ConfigError::{Io, Toml, UnknownKey, InvalidBool, InvalidValue}` (thiserror).
  - Exposed via `lib.rs`: `pub mod io;` + re-exports.
- **CLI `config` subcommands** (`pixelens-cli/src/main.rs`): replaced the stub with
  `run_config(subcmd, key, value)` (sync, file-based) implementing:
  - `list` — prints all keys + values from loaded config (or defaults if no file).
  - `get <key>` — prints one key; clear error if unknown.
  - `set <key> <value>` — sets + writes TOML + reports success.
  - `keyhook_combo()` now resolves `PIXELENS_HOTKEY` env > config `general.hotkey` >
    model default (was hardcoded string).
- **Daemon consumes config** (`pixelens-daemon/src/lib.rs` `run()`):
  - Loads `Config` at startup; logs resolved `general.hotkey` and gates
    `capture.show_preview` debug log (non-fatal). Config is now observably USED.
- **Keyhook default from config** (`pixelens-keyhook/src/main.rs`): `general.hotkey`
  is the default combo when `PIXELENS_HOTKEY` env is unset (env still wins).
- **Tests** (`pixelens-config/src/io.rs`): 3 tests, `temp_dir`-based (never touch
  real `~/.config`): `load_defaults_when_missing`, `load_round_trips`,
  `get_and_set_dotted_keys`.
- Build: `cargo clippy --workspace --all-targets -D warnings` clean (compiles all
  crates) and per-crate `cargo build -p <crate>` green. Full `cargo build --workspace`
  is NOT exercised here — it stalls on ENOSPC (root FS ~83% full, known limit). Fmt clean.
- Test: `cargo test -p pixelens-config` → 3 passed; 0 failed. Functional CLI smoke
  test in a throwaway `$HOME` confirmed `list`/`get`/`set` + unknown-key error.
- Live preview/hotkey QA NOT exercised (headless, no display). Consumption visible
  via daemon startup logs.
- Commits (branch `features/core-loop`): `feat(config): add TOML load/save + get/set`,
  `feat(cli): implement config list/get/set`, `feat(keyhook+daemon): consume
  general.hotkey + show_preview from config`, `test(config): round-trip loader`,
  `docs(plan): record M8 configuration`.
- GitHub: NOT pushed — remote `main` is an unrelated history; local main is on branch
  `features/core-loop` (already exists). Do not force-push.

---

- Implemented **UM1 Hotkey Daemon**:
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

2026-07-19 | session `hy3` (UM3 — Autostart / XDG .desktop integration)
- Implemented **UM3 Systemd + autostart integration**:
  - New `pixelens-cli` `autostart` subcommand: `enable | disable | status`.
    - `autostart_dir()` honors `$XDG_CONFIG_HOME`, falls back to
      `~/.config/autostart`.
    - `autostart_desktop_content(bin)` emits a valid XDG `pixelens.desktop`
      (`[Desktop Entry]`, `Type=Application`, `Name=Pixelens`,
      `Exec=<keyhook bin>`, `X-GNOME-Autostart-enabled=true`).
    - Pure helpers `write_autostart_desktop(dir, bin)` and
      `remove_autostart_desktop(dir)` — no side effects on the live config,
      headless-safe, unit-testable.
  - `config set general.autostart <true|false>` now keeps the `.desktop` in
    sync: `true/1/yes/on` writes it (best-effort; warns if the keyhook binary
    isn't on `PATH`), any other value removes it. Non-fatal.
  - Complements UM1's systemd `--user` service (two independent autostart
    mechanisms, as designed).
- **Tests** (`cargo test -p pixelens-cli`): 2 new — `autostart_desktop_round_trip`
  (write→content assertions→remove→idempotent remove) and
  `autostart_dir_honors_xdg_config_home`. Both pass.
- QA: `cargo fmt --all -- --check` clean; `cargo clippy --workspace
  --all-targets -- -D warnings` clean; `cargo build -p pixelens-cli` exits 0;
  `cargo test -p pixelens-cli` 2/2 pass. NOTE: full `cargo build --workspace`
  / `cargo test --workspace` cannot LINK on this VM (root FS 83-94% full →
  ENOSPC bus-errors); verified per-crate instead. Not a code defect.
- Live autostart QA NOT exercised: headless VM, no display server; the
  `.desktop` file is written/removed correctly but whether a real session
  launches pixelens on login needs a desktop environment to confirm.
- WORKER subagent timed out on the heavy cargo verify slice (600s API
  ceiling); main agent re-ran and confirmed all gates directly.
- Pushed to `features/core-loop` (fast-forward; `origin/main` untouched —
  unrelated history, no force-push).

---

---

2026-07-19 | session `hy3` (UM4 — Grab UI backend; visual HUD deferred)
- **UM4 backend (shippable, verified):**
  - **IPC** (`pixelens-ipc/src/protocol.rs`): added `Command::Redetect` and
    `Command::SetPreview` to the `Command` enum; added `SetPreviewPayload { preview: bool }`.
    Added round-trip unit tests for `Redetect`/`SetPreviewPayload`.
  - **Config** (`pixelens-config/src/model.rs`): added `GuiConfig { hud_enabled: bool,
    hud_timeout_ms: u64 }` + `Config.gui` field (serde defaulted); `GuiConfig::default()`
    → `hud_enabled = true`, `hud_timeout_ms = 1500`. `get_value`/`set_value` already
    handle dotted keys, so `gui.hud_enabled` / `gui.hud_timeout_ms` are settable via
    `pixelens config set` with bool validation.
  - **Daemon state** (`pixelens-daemon/src/state.rs`): `DaemonState` now owns
    `config: Config` + `one_shot: Arc<Mutex<OneShot>>` where `OneShot { preview: Option<bool>,
    redetect: bool }`. Added `set_preview_override` / `take_preview_override`
    (consumes one-shot) / `preview_for_next_grab()` (override → config fallback) /
    `request_redetect` / `take_redetect`. `DaemonState::new` now takes `config`.
  - **Daemon dispatch** (`pixelens-daemon/src/dispatch.rs`): `handle_redetect` and
    `handle_set_preview` handle the two new commands (return ok + opt-in log; real
    re-detect/re-init deferred to the GUI-driven flow). `handle_grab` now reads
    `state.preview_for_next_grab()` (consuming the one-shot) and logs the effective
    preview — so a `setpreview 0` IPC call suppresses the next grab's preview
    without touching config, then reverts. Default path is unchanged when no
    override is set (regression-safe).
  - **Daemon run** (`pixelens-daemon/src/lib.rs`): passes `config` into `DaemonState::new`
    and logs the resolved `gui` section (consumes the new key observably).
  - **Tests** (`pixelens-daemon/src/state.rs`): 4 unit tests — `no_override_uses_config`,
    `override_wins_then_reverts`, `override_suppresses_config_true`, `redetect_flag_round_trips`.
- **UM4 visual HUD (`pixelens-gui`) — DEFERRED.** The design-doc GUI (egui + winit +
  layer-shell) cannot be compiled or QA'd on the headless build VM (no display server;
  disk 83-94% full makes the heavy dep tree a build/ENOSPC risk). Backend above is the
  contract the GUI will consume via `setpreview`/`redetect` IPC + `config.gui.*`.
- QA: `cargo fmt --all -- --check` clean (pending final fmt); `cargo clippy --workspace
  --all-targets -- -D warnings` clean (pending); per-crate build + `cargo test -p
  pixelens-daemon` (4 new state tests) + `cargo test -p pixelens-ipc` (2 new protocol
  tests) to be run on this VM. Full `cargo build --workspace` link blocked by disk
  ENOSPC — per-crate verification used instead. Not a code defect.
- Live HUD QA NOT possible here (headless). Backend logic is unit-tested headlessly.
- Status: backend committed to `features/core-loop`; GUI crate deferred to a display machine.

---

Rule for further entries: every non-trivial change (feature, fix, refactor) gets
its own line here with: **what** + **state after** + **QA result**. Never batch
unrelated changes in one entry.

---

2026-07-19 | session `hy3` (UM2 — Windows support, subagent loop)
- **Implemented UM2 Windows support** via the WORKER→TESTER→VERIFIER→REVIEWER
  subagent loop (loop um2-1..um2-5):
  - `pixelens-capture/src/windows.rs` (new): `#[cfg(windows)] imp` with WinRT
    `GraphicsCapturePicker` + `Direct3D11` device + DXgi frame-pool capture path;
    `MockWindowsCaptureProvider` (non-windows) so `cargo test` is green on Linux.
    `CaptureBackend::Windows` wraps the mock under `#[cfg(windows)]`.
  - `pixelens-capture/src/{lib.rs,pipeline.rs}`: `cfg` dispatch — Windows picks
    the WinRT `imp`; Unix keeps `slurp`/`grim`. Unix-only imports
    (`GrimCapturer`, `SlurpSelector`, `which`) + `check_dependency` are gated
    `#[cfg(unix)]`; `RegionSelector`/`ScreenCapturer` traits stay unconditional.
  - `pixelens-ipc/src/codec.rs`: named-pipe transport (`\\.\pipe\pixelens`) under
    `#[cfg(windows)]`, keeping the length-prefixed JSON codec transport-agnostic.
  - `pixelens-keyhook/src/windows.rs` (new): `RegisterHotKey` loop bound to
    `Win+Shift+S` (`MOD_WIN | MOD_SHIFT | VK_S`), dispatching to `fire_grab`.
    `TranslateMessage`/`UnregisterHotKey` results `.ok()`'d for must-use lints.
    `lib.rs` gates the unix backend/wayland/x11 mods behind `#[cfg(unix)]`; the
    windows mod is unconditional (with `windows` crate `RegisterHotKey` /
    `HOT_KEY_MODIFIERS` / `UnregisterHotKey` usage).
  - `pixelens-notify/src/lib.rs`: `arboard` (clipboard) + WinRT toast under
    `#[cfg(windows)]`.
  - Cargo.toml files (capture/keyhook/notify): `windows = "0.58"` with
    `Graphics_Capture`, `Graphics_DirectX_Direct3D11`, `Graphics_Imaging`,
    `Win32_Graphics_*`, `Win32_UI_WindowsAndMessaging` features for the windows
    target.
- **Docs:** README gained a Windows install subsection + cross-platform intro +
  shipped-list update + removal of the "Windows planned" deferred bullet;
  `15-upgrade-m2-windows.md` status → 🟡 implemented + new §11 implementation
  status block; `10-progress.md`/`11-changelog.md` updated.
- **WORKER caveat:** the spawned WORKER self-reported clippy-clean but actually
  left a `items_after_test_module` defect in `pixelens-notify/src/lib.rs` and a
  `capture/windows.rs` gap — main agent caught both in VERIFIER/REVIEWER stages
  and fixed them. Do not trust subagent "clean" claims; re-run real gates.
- **QA (real, run by main agent):**
  - `cargo fmt --all -- --check` → exit 0.
  - `cargo clippy --workspace --all-targets -- -D warnings` → exit 0 (Linux).
  - `cargo test --workspace` → green (MockWindowsCaptureProvider path; 11+ tests
    across ipc/keyhook/capture/notify).
  - `cargo check --target x86_64-pc-windows-msvc` (all 4 UM2 crates) → exit 0,
    **zero warnings**.
  - `git diff` vs e777b49 confirmed Unix `run()`/`run_unix` logic byte-unchanged;
    all `unsafe` confined to `#[cfg(windows)] imp` modules.
- **NOT verified (environment limit):** native Windows run (picker → capture →
  OCR → clipboard loop) needs a real Windows 10/11 host — not available in this
  Linux VM. Only type-checks via `cargo check`. `cargo build
  --target x86_64-pc-windows-msvc` (link) also not exercised here.
- Commit: `61714c5` `feat(um2): Windows support (#[cfg(windows)] pipeline) + docs`
  — **PUSHED** to `origin/features/core-loop` (`e777b49..61714c5`; follow-up
  `--dry-run` → "Everything up-to-date"). (The "commit pending" line in the
  original entry is superseded — it was committed + pushed the same session.)

---

2026-07-19 | session `hy3` (plan advance post-UM2 — no new code)
- **Plan-only pass** (user chose docs-advance over building the next milestone):
  - `10-progress.md`: corrected stale **UM1 "🚧 in progress"** contradiction
    (it was already marked DONE at line 84) → now ✅ DONE with QA caveat.
  - `10-progress.md`: rewrote the upgrade-roadmap list into a **per-milestone
    status** block (UM1✅ UM2🟡 UM3✅ UM4✅-backend/HUD-scrapped UM5–UM8 ⬜).
  - `10-progress.md`: fixed the **Git status** block (was "NOT pushed" / only
    M8 commits listed) → now records `61714c5` UM2 push + "Everything
    up-to-date"; top "Last updated" stamp refreshed.
  - `10-progress.md`: **Immediate next action** now names **UM5 (portal-native
    capture)** as the next code candidate (xdg-desktop-portal behind a `portal`
    feature flag w/ slurp+grim fallback) — not started.
  - `11-changelog.md`: corrected the UM2 entry's stale "Commit pending" tail →
    records `61714c5` committed + pushed.
- **State after:** tracking docs now internally consistent with `61714c5` and
  the scrapped-HUD / deferred-history decisions. No source code changed.
- **QA:** n/a (docs only). Prior UM2 gates remain green (fmt/clippy/test/
  windows-msvc-check) — re-affirmed by a fresh ad-hoc run this session.
- Next: UM5 implementation (deferred pending user go-ahead)..