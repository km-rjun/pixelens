# 13 — Roadmap: Upgrades & Beyond

This is the roadmap **after v1.0 ships**. Every item must preserve the core flow:
hotkey → select → text copied in <2s, **no menus/confirmations**.

---

## Upgrade M1 — Hotkey daemon (post-v1.0)

**Goal**: bind a global hotkey so `pixelens grab` is no longer CLI-only.

**Work**:
- Add `pixelens-hotkey` crate with xdotool (X11) and virtual-keyboard / 
  systemd user service (Wayland) backend
- Write `pixelens hotkey enable/disable/status` CLI commands
- Create systemd user unit `~/.config/systemd/user/pixelens-hotkey.service`
- Auto-install the unit on first `hotkey enable`

**QA checkpoint** (mandatory before code push):
1. Hotkey triggers `grab` on Wayland **and** X11
2. Selection completes → text appears on clipboard
3. No zombie processes after repeated presses
4. Hotkey daemon does not block or slow the capture loop

**Safety gate before GitHub push**:
- `cargo fmt --check` clean
- `cargo clippy -- -D warnings` clean
- All tests green: `cargo test`

---

## Upgrade M2 — Windows support (replaces Snipping Tool)

**Goal**: Pixelens replaces Windows Snipping Tool. Same 2s text-extract flow, native
to Windows.

**Work**:
- Add `target: windows` to `Cargo.toml`, conditional compilation module
- Windows capture via WinRT screen capture APIs (or GDI+ fallback)
- Windows clipboard via `clipboard` crate + `windows` crate bindings
- Windows notification via `winrt-notification` or PowerShell toast
- MSI/winget packaging draft (M10 tracks this)

**QA checkpoint**:
1. Windows 10/11: hotkey → select → text copied in <2s (ideally ≤1.8s)
2. Clipboard verified via Notepad paste test
3. No Windows Defender false positives / admin rights required
4. Same CLI flags work (`grab`, `status`, `stop`)

**Safety gate**:
- Cross-platform tests: `cargo test --features=windows`
- Review for Windows-specific lint warnings
- Verify no MSVC linker errors on clean build

---

## Upgrade M3 — Systemd service + autostart (user ops)

**Goal**: daemon auto-starts after login, integrates with system lifecycle.

**Work**:
- `pixelens daemon install` / `uninstall` commands
- Install `~/.config/systemd/user/pixelens.service` with socket activation
- Add `Restart=on-failure`, `RestartSec=5`
- `pixelens config autostart` toggle

**QA checkpoint**:
1. Reboot (or `systemctl --user restart pixelens`) → socket active in ≤2s
2. `pixelens grab` works immediately after login (no manual `pixelensd`)
3. Logs visible via `journalctl --user -u pixelens`

**Safety gate**:
- `cargo test --workspace` clean before push
- Review diff: ensure no remote or root escalation in unit file

---

## Upgrade M4 — Grab UI / Actions popup overhaul

**Goal**: Replace the CLI-only grab experience with a minimal HUD that shows
immediate actions (redetect display, toggle preview, grab region) without
leaving the keyboard. Still no menus/confirmations.

**Work**:
- New `pixelens-gui` crate using `iced` or `egui` for instant overlay
- Hotkey + `Space` opens HUD with 3 actions visible for 1.5s
- Mouse/touch optional: keyboard shortcuts `r` (redetect), `p` (preview), `g` (grab)
- Preserve existing slurp+grim flow; HUD only shows options

**QA checkpoint**:
1. HUD appears within 100ms of hotkey+Space
2. `g` keypress goes direct to selection (no visible HUD delay)
3. No input blocking; HUD fades if ignored
4. Timing still meets <2s goal when HUD is skipped (default path unchanged)

**Safety gate**:
- `cargo fmt --check` clean
- HUD integration tests pass on Wayland/X11
- No added unsafe unless absolutely necessary (and documented)

---

## Upgrade M5 — Portal-native capture (optional speedup)

**Goal**: use xdg-desktop-portal directly instead of slurp+grim shell-outs
(~300-400ms faster on some compositors per PRD perf targets).

**Work**:
- Add `pixelens-portal` crate behind feature flag `portal`
- Implement `PortalBackend` satisfying `CaptureBackend`
- Prefer portal if available, fall back to slurp+grim transparently

**QA checkpoint**:
1. Portal path works correctly on wlroots compositors
2. Silent fallback to slurp+grim on GNOME/KDE when portal fails
3. Timing gap to `grab_captured_end_to_end` test narrows by ≥150ms

**Safety gate**:
- Portal path unit-tested with mock portal responses
- Full workspace build + test on clean checkout

**Status (2026-07-20)**: ✅ implemented behind `portal` feature flag.
- New `pixelens-portal` crate (optional, behind `portal` feature). `PortalBackend`
  implements `CaptureProvider`; `capture()` returns the portal outcome directly,
  maps `Cancelled` → `CaptureError::Selector`, and transparently falls back to
  slurp+grim on `Unavailable`/error (non-fatal on decode).
- `pixelens-capture` gains a dep-free `portal` feature enabling the
  `CaptureBackend::Portal(Arc<dyn CaptureProvider + Send + Sync>)` variant.
- Daemon wires `portal_backend` under `#[cfg(feature = "portal")]`
  (`PortalBackend::default()`, no startup `block_on` — no nested-runtime panic).
  Dispatch uses `RawCapture.path` when present (real grim fallback file kept on
  disk), else synthesizes a `portal://` identifier.
- `RawCapture` (in `pixelens-core`) gained `path: Option<PathBuf>` so a capture
  may carry its on-disk file.
- **Real-portal capture (PipeWire ScreenCast) deferred to M3 of portal work** —
  `RealPortalSession::run` currently returns `Unavailable` (falls through to the
  slurp/grim fallback), so v1 behavior is preserved until pipewire wiring lands.
- QA: fmt + clippy `--all-targets -D warnings` clean (default + portal);
  daemon tests 4/4 (default) + 4/4 (portal); portal crate unit 3/3 (mock/
  cancel/fallback); `cargo check --target x86_64-pc-windows-msvc` clean for
  `pixelens-capture`. Native portal run (wlroots) NOT exercised headless.

---

## Upgrade M6 — Multi-display smart selection

**Goal**: detect which monitor the cursor is on; limit slurp to that output.

**Work**:
- Extend `slurp -g <output>` in `SlurpSelector` when cursor position known
- Add `wayland-protocols` dependencies for cursor-seat queries
- Document per-display capture in CLI `--help`

**QA checkpoint**:
1. Multi-monitor setup: selection region stays on correct monitor
2. Single monitor: behavior unchanged
3. No regressions to single-display timing

**Safety gate**:
- `cargo clippy -- -D warnings` clean
- One-shot integration test on Wayland multi-head VM

---

## Upgrade M7 — On-demand vs continuous mode

**Goal**: user can toggle whether Pixelens stays resident or starts fresh each
time (privacy-focused minimal footprint).

**Work**:
- Add `pixelens config mode = "ondemand"` (default, exit after clip) or
  `"continuous"` (keep in RAM for repeated grabs)
- When `ondemand`, daemon exits after each successful OCR + clipboard write
- When `continuous`, daemon stays warm (current default)

**QA checkpoint**:
1. `mode = "ondemand"` → daemon exits after grab; next grab spawns fresh
2. Process tree shows clean exit (no orphan)
3. Timing still <2s in ondemand (Tesseract warm-start cached)

**Safety gate**:
- Explicit config file lock and validation check
- Test both modes in isolation (`--features=continuous` / `--features=ondemand`)

---

## Upgrade M8 — Tesseract performance tuning

**Goal**: drop OCR latency toward the PRD target (v1 requires ≤1.8s, upgrades
aim for ≤1.6s median).

**Work**:
- Pre-warm Tesseract on daemon startup with dummy image
- Enable `--oem 1` (LSTM only) in `TesseractOcrEngine`
- Add `--psm 7` for single text line, `--psm 6` for block

**QA checkpoint**:
1. 5x repeated `pixelens grab` on same text: avg latency ≤1.6s
2. Accuracy unchanged on varied fonts (English, monospace, PDFs)
3. Memory footprint ≤50MB after warm (measured via `ps aux`)

**Safety gate**:
- Benchmark script in `/scripts/bench.sh` runs and passes
- No unsafe code added in OCR path

---

## Definition of "upgrades done"

1. All upgrade milestones reach QA+green build
2. CHANGELOG.md updated for each milestone (not just git log)
3. Each upgrade merged via atomic commit, reviewed, pushed only after safety gates pass

---

**Next action (when you're ready)**: pick an upgrade, and I'll create its milestone slice with the exact PRD alignment. Do not deviate from "no menus / no cloud / no AI" — everything serves that 2-second text-extract flow.