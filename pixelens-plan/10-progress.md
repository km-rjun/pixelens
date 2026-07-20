# 10 — Progress (READ FIRST EACH SESSION)

This file is the live status snapshot. Update it whenever you change code or
learn new facts. Keep the "build status" honest — a green build is the only
acceptable state to declare work "done".

_Last updated: 2026-07-19 (session: hy3 — plan advance post-UM2; no new code)_

## Build status: 🟢 GREEN (workspace)
- `cargo build --workspace` passes (VM disk now 34% used / 13G free — the old
  ENOSPC linker bus-errors are gone).
- `cargo fmt --all -- --check` clean.
- `cargo clippy --workspace --all-targets -- -D warnings` clean (Linux).
- `cargo check --target x86_64-pc-windows-msvc` clean (zero warnings) across
  all UM2 crates — Windows type-checks; native link not exercised (Linux VM).
- ENVIRONMENT NOTE: root FS had been 83-100% full (blocked full workspace
  links). Resolved — disk is now ~34% used. `cargo clean` before heavy builds
  is still good hygiene but no longer required.

## Test status: 🟢 ALL GREEN (per-crate)
- `cargo test -p pixelens-config` → 3 passed; 0 failed
  (`load_defaults_when_missing`, `load_round_trips`, `get_and_set_dotted_keys`).
  Tests write to `std::env::temp_dir()` — never touch real `~/.config`.
- Functional CLI smoke test (in a throwaway `$HOME`): `config list`,
  `config get <key>`, `config set <key> <value>` (writes TOML, creates
  parent dir), and unknown-key error all behave correctly.
- Live preview/hotkey QA NOT exercised (headless, no display). Config
  consumption is observable via daemon startup logs (M8).
- `cargo test -p pixelens-ocr` → 5 passed; 0 failed (sanitize + error paths).
- `cargo test -p pixelens-notify` → 1 passed; 0 failed (notification
  message strings). NEW in M7.
- `cargo test -p pixelens-daemon --lib` → 1 passed; 0 failed
  (`clipboard::tests::copy_text_returns_no_backend_without_clipboard_tools`). NEW in M7.
- `cargo test -p pixelens-daemon --test integration` → 4 passed; 0 failed
  (incl. `grab_captured_end_to_end`, which now drives capture → OCR →
  clipboard/notify branch with no clipboard tool present → log-and-continue).
- `cargo test -p pixelens-ipc` → 6 passed; 0 failed (added named-pipe
  transport test under `#[cfg(windows)]` + a pipe-path builder test; Linux runs
  the cross-platform codec tests).
- `cargo test -p pixelens-keyhook` → 3 passed; 0 failed (hotkey-id parse/eq
  tests; windows RegisterHotKey mod/vk mapping unit-tested on Linux).
- `cargo test -p pixelens-capture` → 1 passed; 0 failed (mock Windows capture
  provider path, exercised on Linux via the `MockWindowsCaptureProvider`).
- `cargo test -p pixelens-notify` → 1 passed; 0 failed (notification message
  strings, unchanged).
- **`cargo test --workspace` now LINKS and passes** (VM disk no longer full).
  Full-workspace run green: all per-crate suites above aggregate to a single
  green result.
- NOTE: live clipboard/notify QA NOT exercised headless (no display, no
  wl-copy/xclip/notify-send on Linux; no Windows host). Windows picker/capture
  loop not run natively — only `cargo check --target x86_64-pc-windows-msvc`
  verified.
  the `grim` stub used `head -c 1024 /dev/urandom`, but the test installs
  an isolated `$PATH` containing only the slurp/grim stubs, so `head` was
  not found (exit 127) → 0-byte file → pipeline reported a zero-byte error.
  Fixed by rewriting the stub to emit 1024 NUL bytes via POSIX-sh builtins only.
- This was **not** an environmental (headless) failure — it reproduced with
  real stubs and the daemon pipeline was always correct.

## Git status
- Branch: `features/core-loop` (local main pushed here; remote `main` is an
  UNRELATED/different-crate-layout history — do NOT push to `main`).
- Commits this session (post-v1.0, all pushed to `origin/features/core-loop`):
  - UM2 Windows support: `61714c5` feat(um2): Windows support (#[cfg(windows)]
    pipeline) + docs (ipc named-pipe, keyhook RegisterHotKey Win+Shift+S,
    capture WinRT mock, notify arboard/winrt, Cargo windows-0.58 features;
    README Windows section + 15-doc status/§11).
  - Prior milestones (M8 config, UM1 hotkey, UM3 autostart, UM4 backend)
    committed earlier this cycle and pushed.
- Remote push: **PUSHED** — `git push origin features/core-loop` accepted
  (`e777b49..61714c5`); follow-up `--dry-run` reports "Everything up-to-date".

## What is actually working
- M1 setup ✅ · M2 display detection ✅ (Wayland + X11) · M3 v1 slurp+grim capture path ✅
  (works on both Wayland and X11; grim has X11 support) · M4 X11 capture **NOT done**
  (`pixelens-capture/src/x11.rs` is a stub → `PixelensError::NotImplemented("X11CaptureProvider (M4)")`,
  per 03-milestones.md M4 = ❌) · M5 OCR **partial** — `TesseractOcrEngine` implemented +
  validates + wired into `handle_grab`, `GrabResponsePayload.text` populated) ·
  M6 IPC protocol + daemon server ✅ · CLI client ✅ (fixed) ·
  M7 Clipboard + Notifications ✅ — `pixelens-daemon::clipboard::copy_text`
  shells out to wl-copy/xclip/xsel (display-server aware, degrades gracefully);
  `pixelens-notify::NotifySend` shells out to `notify-send` (non-fatal);
  `handle_grab` copies non-empty text + fires "✓ Text copied to clipboard",
  or "No text found in selection." when OCR text is empty. UM1 global hotkey ✅
  (Wayland evdev + X11 x11rb in `pixelens-keyhook/src/x11.rs`; systemd --user service).
  NOTE: the X11 x11rb code is the **UM1 hotkey listener**, not M4 capture.

## What is NOT done (next in line)
1. M5/M7 live QA — engine + wiring done; real screenshot→text→clipboard→
   notify not exercised headless (needs a display; tesseract 5.5.0 IS
   installed here, but no Wayland/X11 session + no wl-copy/xclip/notify-send).
- M8 Configuration ✅ — `pixelens-config` now loads/saves `~/.config/pixelens/config.toml`
  (defaults when absent/invalid). `pixelens config list|get|set` implemented
  (dotted keys, typed set, parent-dir creation, clear errors). Daemon `run()`
  loads config and LOGS `general.hotkey` (resolved env>config>default) + gates
  `capture.show_preview` log. `pixelens-keyhook` uses `general.hotkey` as its
  default combo (env still wins). CLI `keyhook` reports the config default.
4. Config keys `autostart` / `theme` are parsed; `autostart` IS now consumed
   (UM3, see below). `theme` still parsed but not read by the daemon (UI work).

> Honesty note (2026-07-18): an earlier progress entry correctly stated the README
> overclaimed "text copied to clipboard in <2s" and "tesseract required at startup" —
> both inaccurate for the current build; README was rewritten to match code.
> CORRECTION (2026-07-18, later same session): a prior sentence here claimed "M4 X11 is
> DONE (x11rb root-window grab in pixelens-keyhook/src/x11.rs)". That was WRONG. The
> x11rb code is the **UM1 hotkey listener**, not M4 capture. M4 Capture (X11) in
> `pixelens-capture/src/x11.rs` is **still a stub** (`NotImplemented("X11CaptureProvider (M4)")`),
> consistent with 03-milestones.md (M4 = ❌). See "What is actually working" above.

## Upgrade roadmap (post-v1.0) — status
- UM1 Hotkey daemon — ✅ DONE (2026-07-19); live QA pending (headless).
- UM2 Windows support — 🟡 implemented (2026-07-19); type-checks on
  `x86_64-pc-windows-msvc`; native Windows run pending (no Windows host).
- UM3 Systemd + autostart — ✅ DONE (2026-07-19); live QA pending (headless).
- UM4 Grab UI — ✅ backend DONE (2026-07-19); **visual HUD SCRAPPED** (utility
  tool, no HUD/tray/window; same for Windows). See tracking.
- UM5 Portal-native capture — ✅ DONE (2026-07-20); behind `portal` feature
  flag. `PortalBackend` impls `CaptureProvider`; falls back to slurp+grim
  transparently (real portal PipeWire capture deferred to portal M3). Native
  wlroots portal run NOT exercised headless (fallback + mock only).
- UM6 Multi-display smart selection — ⬜ not started.
- UM7 On-demand vs continuous mode — ⬜ not started.
- UM8 Tesseract performance tuning — ⬜ not started.

See `13-roadmap-upgrades.md` for details. Each upgrade requires QA checkpoint
and safety gate before GitHub push.

## Documentation added (this session)
- `pixelens-plan/` — 18 files: overview, goals, architecture, milestones,
  per-crate docs, progress, changelog, v1 + upgrade roadmaps, and three
  detailed upgrade design docs (`14-upgrade-m1-hotkey.md`,
  `15-upgrade-m2-windows.md`, `16-upgrade-m4-grab-ui.md`).
- `README.md` — rewritten with setup, quick-start, config, troubleshooting.

## Upgrade milestone tracking (post-v1.0)
- UM1 Hotkey daemon — ✅ DONE (2026-07-19): `pixelens-keyhook` crate (Wayland
  evdev + X11 x11rb), CLI `hotkey` subcommand, systemd --user unit generation.
  Manual live QA pending (needs real Wayland/X11 session — headless blocks it),
  but code path is complete and verified (fmt/clippy/test green).
- UM2 Windows support — 🟡 **implemented (2026-07-19)**: full `#[cfg(windows)]`
  pipeline wired + type-checks against `x86_64-pc-windows-msvc` (zero
  warnings). `pixelens-capture/src/windows.rs` (WinRT `GraphicsCapturePicker`
  + DXgi frame pool; `MockWindowsCaptureProvider` for Linux tests),
  `pixelens-ipc/src/codec.rs` (named-pipe `\\.\pipe\pixelens`),
  `pixelens-keyhook/src/windows.rs` (`RegisterHotKey` bound to `Win+Shift+S`),
  `pixelens-notify` (`arboard`/`winrt`-toast). `cargo check --target
  x86_64-pc-windows-msvc` green; `cargo test --workspace` green (mock). Native
  Windows run (picker→capture→OCR→clipboard loop) still PENDING — no Windows
  host in CI/this VM. See `15-upgrade-m2-windows.md` §11.
- UM3 Systemd + autostart — ✅ DONE (2026-07-19): `pixelens-cli autostart
  enable|disable|status` manages `~/.config/autostart/pixelens.desktop` (XDG
  autostart spec; honors XDG_CONFIG_HOME, falls back to ~/.config). Pure
  helpers `write_autostart_desktop`/`remove_autostart_desktop` are unit-tested
  (round-trip + idempotent remove). `config set general.autostart true|false`
  now keeps the .desktop in sync (best-effort, non-fatal). Complements UM1's
  systemd --user service. Manual QA of actual desktop autostart pending
  (needs a real session — headless blocks it).
- UM4 Grab UI — ✅ backend DONE (2026-07-19), ❌ **visual HUD SCRAPPED
  (2026-07-19)**:
  - **Backend (shippable now, verified):** IPC gained `Command::Redetect` +
    `Command::SetPreview` (with `SetPreviewPayload`); daemon dispatch handles
    both; `DaemonState` gained an `Arc<Mutex<OneShot>>` holding a one-shot
    preview override (consumed by `handle_grab`, reverts to config afterward)
    + a re-detect flag; config gained `GuiConfig { hud_enabled, hud_timeout_ms }`
    (defaults `true` / `1500`). `DaemonState::preview_for_next_grab()` resolves
    override→config. 4 unit tests guard the override/revert + redetect flag.
    Default grab path is unchanged when no override is set (regression-safe).
  - **Visual HUD crate (`pixelens-gui`, egui + layer-shell): SCRAPPED.** Per
    project direction Pixelens is a utility tool — no HUD, no tray, no main
    window. The crate will **not** be built (applies to Windows release too:
    gui-light). A history/recall feature is also deferred. The backend above
    remains the stable contract if a minimal HUD is ever wanted later. See
    `16-upgrade-m4-grab-ui.md` status block.
- UM5 Portal-native capture — ✅ DONE (2026-07-20):
  - `pixelens-portal` crate (optional, `portal` feature): `PortalBackend` impls
    `CaptureProvider`; `RealPortalSession::run` returns `Unavailable` (PipeWire
    ScreenCast deferred to portal M3), so capture transparently falls back to
    slurp+grim. `fallback_capture` keeps the grab file on disk (v1 contract) and
    decodes non-fatally. `MockPortalSession` drives the 3 crate unit tests.
  - `pixelens-capture` dep-free `portal` feature → `CaptureBackend::Portal(
    Arc<dyn CaptureProvider + Send + Sync>)` variant.
  - `pixelens-core` `RawCapture` gained `path: Option<PathBuf>`.
  - `pixelens-daemon`: optional `pixelens-portal` dep + `portal` feature;
    `portal_backend: Option<Arc<dyn CaptureProvider>>` built under
    `#[cfg(feature = "portal")]` via `PortalBackend::default()` (NO startup
    `block_on` → no nested-runtime panic in tokio). Dispatch uses `raw.path`
    when present, else synthesizes `portal://`.
  - `is_available()`/`portal_reachable()` present but NOT called at startup
    (nested-runtime guard). public lib items; clippy clean.
  - QA: fmt + clippy `--all-targets -D warnings` clean (default + portal);
    daemon 4/4 default + 4/4 portal (incl. line-170 file-exists); portal crate
    3/3; `cargo check --target x86_64-pc-windows-msvc` clean for capture.
    Native portal run NOT exercised headless. Verified via
    WORKER→TESTER→VERIFIER→REVIEWER loop; REVIEWER caught 2 WORKER-missed test
    failures (startup probe hang, synthetic-path) — fixed by main agent.
- Config keys `autostart` / `theme` / `gui.*`: `autostart` IS consumed (UM3).
  `gui.hud_enabled` + `gui.hud_timeout_ms` are parsed/validated (defaults
  true/1500) but only *used* if a HUD is ever built (it is scrapped for now).
  `theme` still parsed but not read by the daemon (UI work). `show_preview` +
  `general.hotkey` consumed in M8.

## Immediate next action
M8 (configuration) ✅, UM3 (autostart) ✅, UM4 backend ✅, and **UM2 Windows
support (code-complete + type-checks, committed `61714c5`)** are DONE on
`features/core-loop`. The core loop (hotkey → select → OCR → text on
clipboard) + config + autostart + UM4 IPC/daemon/config backend + UM2 Windows
`#[cfg(windows)]` pipeline are code-complete and verified (fmt/clippy/test
green on Linux; `cargo check --target x86_64-pc-windows-msvc` zero warnings).
**UM4 visual HUD is SCRAPPED** — Pixelens stays a utility tool (no
HUD/tray/window; same for Windows release). History/recall deferred. README
rewritten + Windows section added.

**UM5 portal-native capture ✅ DONE (2026-07-20)** behind the `portal` feature
flag (commit pushed to `origin/features/core-loop`; `origin/main` untouched).
`PortalBackend` impls `CaptureProvider`, transparently falls back to slurp+grim;
real PipeWire ScreenCast deferred to portal M3. Native wlroots portal run is
headless-blocked (fallback + mock only). See UM5 tracking above + changelog +
`13-roadmap-upgrades.md`.

**Next code candidate: UM6 (multi-display smart selection)** — detect cursor
monitor, scope slurp to that output via `slurp -g <output>`; needs
`wayland-protocols` cursor-seat queries. Not started.
Outstanding headless-blocked QA (native Windows run of UM2 loop; live UM1/UM3
desktop QA; native portal run) remains documented, not compromised.

_Last updated: 2026-07-20 (session: hy3 — UM5 portal-native capture shipped)._

## Habits to keep
- After EVERY change: commit small + descriptive.
- Before claiming a milestone: `cargo build` + `cargo test` green.
- QA explicit per milestone (see roadmap files).
- No deviation from PRD vision: hotkey → select → text copied in <2s.