# 15 — Upgrade M2: Windows Support (detailed design)

**Depends on**: v1.0 core loop + Upgrade M1 (hotkey). Windows must **replace
the Snipping Tool** — i.e. the same `Win+Shift+S` muscle memory triggers
Pixelens instead.
**Status**: 🟡 implemented (compiles + type-checks on `x86_64-pc-windows-msvc`; native Windows run pending) · **Owner**: odin.

---

## 1. Problem statement

Windows users have no `slurp`/`grim`. The native equivalent is the Snipping
Tool / Snip & Sketch bound to `Win+Shift+S`. Pixelens should provide a
drop-in: same key chord, same result (text on clipboard in <2s), no cloud,
no account.

## 2. Constraints (PRD-preserving)

- No AI, no cloud, no accounts, no history, no menus.
- Hotkey → select → text copied, under 2s (target ≤1.8s to beat Snipping Tool
  latency perception).
- CLI flags `grab`, `status`, `stop` must work identically to Linux.
- Must not require admin rights or trigger Windows Defender smartscreen on a
  normal install.

## 3. Architecture

```
pixelens.exe (CLI, same IPC model)
pixelensd.exe (daemon, Windows named pipe instead of Unix socket)
pixelens-keyhook.exe (global hotkey via RegisterHotKey, bound to Win+Shift+S)
        │
        ▼  (named pipe: \\.\pipe\pixelens)
pixelensd.exe ──► capture (WinRT / GDI+) ──► OCR (Tesseract) ──► clipboard + toast
```

IPC swaps Unix socket → Windows named pipe. The `pixelens-ipc` codec
(length-prefixed JSON) is transport-agnostic; only the bind/connect primitives
change behind `#[cfg(windows)]`.

## 4. Crate / target changes

- `Cargo.toml`: `[target.'cfg(windows)'.dependencies]` block:
  - `windows = { version = "0.58", features = ["Win32_..."] }`
  - `clipboard-win` or `arboard` (cross-platform clipboard — prefer `arboard`,
    already usable on Linux too; unify later).
  - `winrt-notification` for toasts.
- New module `pixelens-capture::windows` with `GrabPipeline` impl using:
  - **Preferred**: `Windows.Graphics.Capture` (WinRT screen capture) via
    `windows` crate — clean, no GDI bitmaps in RAM longer than needed.
  - **Fallback**: `BitBlt` / `PrintWindow` via GDI+ if WinRT capture
    unavailable (older builds).
- Selection UI: WinRT `GraphicsCapturePicker` gives a native region selector
  that looks identical to Snipping Tool — **use it**; do not build a custom
  overlay (keeps the "no menu" feel and matches user expectation).

## 5. Hotkey on Windows

`RegisterHotKey(NULL, id, MOD_WIN | MOD_SHIFT, VK_S)` in `pixelens-keyhook`
compiled for windows. On press → connect named pipe → `Grab`. This directly
**replaces** Snipping Tool if the user disables the OS binding (documented in
README: *Settings → Ease of Access → Keyboard → turn off Snip & Sketch*).

## 6. Clipboard & notify

- Clipboard: `arboard::Clipboard::set_text(text)` (cross-platform, same call
  site as Linux eventually).
- Notify: `winrt-notification::Toast` with "✓ Text copied to clipboard".

## 7. Packaging (draft; finalised in M10/Upgrade)

- `cargo build --target x86_64-pc-windows-msvc` → `pixelens.exe`,
  `pixelensd.exe`, `pixelens-keyhook.exe`.
- `cargo wix` or `cargo tauri` not used; hand-written `pixelens.wxs` (WiX) or
  a simple Inno Setup script. winget manifest `Pixelens.yaml` submitted to
  microsoft/winget-pkgs.
- Code-signing: recommend a cheap Authenticode cert to avoid SmartScreen
  warnings (noted as optional-but-strongly-recommended).

## 8. Files touched (cross-platform guards)

| File | Change |
|------|--------|
| `Cargo.toml` | windows target deps |
| `pixelens-capture/src/windows.rs` (new) | `GrabPipeline` impl |
| `pixelens-capture/src/lib.rs` | `cfg` dispatch on `target_os` |
| `pixelens-ipc/src/codec.rs` | named-pipe transport under `cfg(windows)` |
| `pixelens-keyhook/src/windows.rs` (new) | `RegisterHotKey` loop |
| `pixelens-notify/src/lib.rs` | `winrt-notification` under cfg |
| `README.md` | Windows install + Snipping Tool replacement steps |

## 9. QA checklist (mandatory before push)

1. Windows 10/11: `Win+Shift+S` (after disabling OS snip) → region picker →
   text on clipboard in <2s (measure, target ≤1.8s).
2. Notepad paste test confirms clipboard content.
3. `pixelens.exe grab` / `status` / `stop` behave identically to Linux.
4. No admin prompt on normal install; no Defender quarantine on a signed build.
5. Toast appears on success; "No text found" toast on empty selection.

## 10. Safety gate before GitHub push

- `cargo test --features=windows` green (mock capture backend for CI).
- `cargo clippy -- -D warnings --target x86_64-pc-windows-msvc` clean (or
  document why a lint is allowed).
- No `unsafe` in hotkey/clipboard paths unless wrapped + commented.
- MSVC linker build succeeds on a clean checkout (CI matrix adds windows).

## 11. Implementation status (2026-07-19)

What shipped in the `features/core-loop` UM2 commit:

- **`pixelens-capture/src/windows.rs`** (new): `GrabPipeline` `#[cfg(windows)]`
  `imp` with the WinRT `GraphicsCapturePicker` + `Direct3D11` device + DXgi
  frame-pool path, plus a `MockWindowsCaptureProvider` (used on non-Windows so
  `cargo test` stays green on Linux). `CaptureBackend::Windows` wraps the mock
  under `#[cfg(windows)]`.
- **`pixelens-capture/src/lib.rs` / `pipeline.rs`**: `cfg` dispatch —
  Windows builds pick the WinRT `imp` path; Unix keeps `slurp`/`grim`. Unix
  imports (`GrimCapturer`, `SlurpSelector`, `which`) and `check_dependency`
  are gated `#[cfg(unix)]`; the `RegionSelector`/`ScreenCapturer` traits stay
  unconditional (the mock provider implements them).
- **`pixelens-ipc/src/codec.rs`**: named-pipe transport
  (`\\.\pipe\pixelens`) under `#[cfg(windows)]`, keeping the length-prefixed
  JSON codec transport-agnostic.
- **`pixelens-keyhook/src/windows.rs`** (new): `RegisterHotKey` loop bound to
  `Win+Shift+S` (`MOD_WIN | MOD_SHIFT | VK_S`), dispatching to the shared
  `fire_grab`. `TranslateMessage`/`UnregisterHotKey` results are `.ok()`'d to
  satisfy must-use lints.
- **`pixelens-notify/src/lib.rs`**: `arboard` (clipboard) + WinRT toast under
  `#[cfg(windows)]`.
- **Cargo.toml files**: `windows = "0.58"` with the `Graphics_Capture`,
  `Graphics_DirectX_Direct3D11`, `Graphics_Imaging`, `Win32_Graphics_*`, and
  `Win32_UI_WindowsAndMessaging` features added to the windows target.

Verification actually run (not assumed):

- `cargo fmt --all -- --check` → exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0 (Linux).
- `cargo test --workspace` → green (mock Windows backend on Linux).
- `cargo check --target x86_64-pc-windows-msvc` (all 4 UM2 crates) → exit 0,
  **zero warnings**.
- All `unsafe` blocks confined to `#[cfg(windows)] imp` modules.

Outstanding (matches the original QA checklist §9 — deferred, not done):

- **Native Windows run not yet performed** (CI is Linux-only; the VM has no
  display server). The picker → capture → OCR → clipboard loop must be
  confirmed end-to-end on a real Windows 10/11 host before calling UM2
  shipping-complete. Latency target ≤1.8s to beat the Snipping Tool
  perception.
- Linker build (`cargo build --target x86_64-pc-windows-msvc`) not exercised
  here — only `cargo check` (type-check, no link) is verifiable on Linux.
- `cargo test --features=windows` CI matrix (§10.1) not added yet (no Windows
  runner in this environment).
