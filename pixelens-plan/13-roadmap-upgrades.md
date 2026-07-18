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

## Upgrade M2 — Portal-native capture (optional speedup)

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

## Upgrade M4 — Multi-display smart selection

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

## Upgrade M5 — On-demand vs continuous mode

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

## Upgrade M6 — Tesseract performance tuning

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