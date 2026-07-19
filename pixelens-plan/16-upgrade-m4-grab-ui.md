# 16 — Upgrade M4: Grab UI / Actions Popup Overhaul (detailed design)

**Depends on**: v1.0 + Upgrade M1 (hotkey) + Upgrade M3 (autostart). The HUD
is an *enhancement* to the grab trigger, not a replacement of the core flow.
**Status**: 📋 planned · **Owner**: odin.

> **Implementation status (2026-07-19):**
> - ✅ **UM4-core — backend/IPC/config — DONE & verified.** `Command::Redetect`
>   and `Command::SetPreview` added to the IPC protocol; daemon dispatch handles
>   both; `DaemonState` gained a `OneShot` override (one-shot preview that reverts
>   after the next grab) + a re-detect flag; config gained `GuiConfig { hud_enabled,
>   hud_timeout_ms }` (defaults `true` / `1500`). Regression-guarded by unit tests
>   on `DaemonState`. Default grab path is byte-for-byte unchanged when no override
>   is set.
> - ⏸️ **UM4-gui — the visual HUD crate (`pixelens-gui`) — DEFERRED.** The proposed
>   `egui` + `winit` + `wlr-layer-shell` surface **cannot be compiled or QA'd on the
>   headless build VM** (no display server, no Wayland/X11, disk 83-94% full → heavy
>   dep tree risks ENOSPC on every future build). It will be implemented on a machine
>   with a real display. The backend above is the contract the GUI will consume via
>   `setpreview` / `redetect` IPC + `config.gui.*`.

---

## 1. Problem statement

Today grab is either CLI or a bare hotkey → slurp overlay. There is no quick way
to adjust *how* a grab happens (which display, preview on/off, re-detect) without
editing config or typing flags. Users expect a lightweight "actions" surface
like Flameshot's sidebar. But the PRD forbids menus/confirmations in the
**default** path. So: HUD is **opt-in, keyboard-first, auto-dismissing**, and
never sits between the hotkey and the grab unless the user explicitly opens it.

## 2. Constraints (PRD-preserving — critical)

- Default path stays: hotkey → slurp overlay → text copied. **Zero added
  latency, zero new clicks.**
- No persistent menu, no tray-by-default (tray is Upgrade M9 and optional).
- HUD must not block input or capture focus from slurp.
- Keyboard-first: every HUD action has a key. Mouse is secondary.

## 3. Interaction model

| Input | Result |
|-------|--------|
| Hotkey (e.g. `Super+Shift+S`) | immediate `grab` (unchanged, <2s) |
| Hotkey **+ hold `Space`** (or `Super+Shift+Space`) | opens HUD for ~1.5s |
| In HUD: `g` | grab now (same as default) |
| In HUD: `r` | re-detect display / re-query outputs |
| In HUD: `p` | toggle `show_preview` for this grab only |
| HUD ignores input / times out | dismisses, no side effect |

The HUD is a **transient overlay**, not a window that steals focus. It renders
as a small unfocused layer (wlr-layer-shell on Wayland, a `NETWM` tooltip on
X11) that fades after 1.5s or on any grab/cancel.

## 4. New crate: `pixelens-gui`

```
pixelens-gui/
├── Cargo.toml
└── src/
    ├── lib.rs          # Hud struct, show(actions) -> HudEvent
    ├── wayland.rs      # wlr-layer-shell surface (unfocused, top layer)
    ├── x11.rs          # override-redirect tooltip window
    └── keys.rs         # tiny key listener (reuse pixelens-keyhook backend)
```

Dependency choice: **`egui`** (immediate mode, trivial to render a 3-button
HUD, no retained-state bugs) over `iced` (more boilerplate, better for full
apps). The HUD is 3 buttons + labels — egui is the right weight. Add
`egui = "0.29"`, `egui-winit` + `winit` for the event loop, and a Wayland
backend via `egui` + `smithay-client-toolkit` OR just render to a
`wlr-layer-shell` surface we manage. **Decision**: keep it minimal — render the
HUD with `egui` into a `winit` window configured as a layer-shell surface on
Wayland (via `wayland-client` + `wlr-layer-shell` protocol) and an
override-redirect window on X11. No float, no decorations.

## 5. Integration with grab flow

The HUD lives in `pixelens-keyhook` (or a new `pixelens-hud` binary spawned by
the keyhook). On `Space` hold:
1. keyhook spawns `pixelens-gui` (or sends an IPC `Hud` command to daemon).
2. HUD shows; user presses `g`/`r`/`p`.
3. `g` → daemon `Grab` (same path). `p` → flips a one-shot flag the daemon
   reads when building the pipeline. `r` → daemon re-runs display detection.
4. HUD exits; slurp overlay takes over.

The daemon needs **one new IPC command**: `ConfigOverride { preview: bool }`
(one-shot) and `RedetectDisplay`. Both already fit the `Command` enum pattern;
add `Command::Redetect` and `Command::SetPreview`.

## 6. Config additions

```toml
[gui]
hud_enabled = true        # master switch for the HUD feature
hud_timeout_ms = 1500
```

If `hud_enabled = false`, the `Space` chord is ignored and behavior is identical
to v1.0.

## 7. Files touched

| File | Change |
|------|--------|
| `Cargo.toml` | add `pixelens-gui` member + dep; `egui` etc. |
| `pixelens-gui/` (new) | whole crate |
| `pixelens-ipc/src/protocol.rs` | `Command::Redetect`, `Command::SetPreview` |
| `pixelens-daemon/src/dispatch.rs` | handle the two new commands |
| `pixelens-daemon/src/state.rs` | one-shot preview flag + redetect |
| `pixelens-config/src/model.rs` | `GuiConfig` |
| `README.md` | HUD keys documented |

## 8. QA checklist (mandatory before push)

1. Hotkey alone → grab in <2s, HUD never appears (latency unchanged).
2. Hotkey + `Space` → HUD visible within 100ms.
3. `g` in HUD → immediate slurp overlay (no extra delay).
4. `p` toggles preview for that one grab (verify file/notification differs).
5. `r` re-detects display (log shows re-query).
6. HUD auto-dismisses after timeout with no side effect; never steals slurp
   focus.
7. `gui.hud_enabled = false` → `Space` chord is a no-op.

## 9. Safety gate before GitHub push

- `cargo fmt --check` + `cargo clippy -- -D warnings` clean.
- HUD integration test: spawn `pixelens-gui` headless, assert it exits on
  timeout and on `g`.
- No `unsafe` in the GUI event loop; any FFI to wlr-layer-shell is isolated
  and documented.
- Confirm default grab path is bit-for-bit unchanged (add a regression test
  that times `grab` with HUD disabled).
