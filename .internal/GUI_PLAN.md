# GUI Plan — Pixelens (Draft)

Status: planning only. No implementation started.

## Constraints

- Rust-first. No React, TypeScript, Vite, Tauri, or other frontend frameworks unless a real UI requirement is evaluated and documented in AGENTS.md/roadmap.
- Native Rust GUI overlay/panel selected by user.
- First platform target: Linux Wayland.
- Primary purpose not yet finalized; user will define in detail later.
- Allowed modularization: add one optional crate only after documenting why. Prefer focused modules unless a separate GUI process is justified.
- AGENTS.md and docs/PROJECT_STATE.md remain authoritative for project identity and command semantics.

## Candidate Placement (Not Final)

Option A: keep GUI as a module under an existing crate, e.g. `pixelens-menu` or `pixelens-daemon`, to preserve crate count.

Option B: add one optional crate (e.g. `pixelens-gui`) only if it needs:
- independent lifecycle/build feature,
- OS-specific GUI dependencies,
- separate process/window runtime,
- clean optional dependency gating.

Do not create the crate until the above justification is written and approved.

## Open Design Items

- Exact visual layout and interaction model.
- Whether GUI replaces or supplements compositor keybindings.
- Whether the action bar becomes persistent/panel vs transient overlay.
- Settings editing location and validation.
- Daemon vs CLI ownership of GUI lifecycle.
- Cross-platform strategy after Wayland.

## Immediate Next Step

User will provide detailed GUI requirements. Then convert this draft into a concrete architecture and roadmap before code changes.

## Hotkey / Pie Menu Concept

Trigger:
- Normal press: existing behavior (selection + default command).
- Hold for threshold (default 1.5s): open GUI circular pie menu at the current cursor location.

Visual model:
- Circular disc, not bar.
- Sectors around center correspond to available actions.
- Center area is an empty cancel zone.
- Closing default behavior if cursor leaves/does not select an action before release.

Behavior:
- Once open, hovering a sector highlights/selects it.
- Releasing or selecting acts on OCR/text from current/previous selection as defined by command flow.
- Cancel occurs if selection ends in center/cancel sector or times out.

External binding model:
- Use compositor press/release bindings, not in-process global grabs:
  `bind = ..., exec, pixelens hotkey press <action>`
  `bindr = ..., exec, pixelens hotkey release <action>`
- Daemon records press timestamps and opens pie menu only after threshold elapses.
- Normal release before threshold runs the immediate action.
- This preserves AGENTS recommendations (compositor owns keys) while enabling held-menu behavior.

Open technical questions:
- What is the exact trigger source? Global hotkey daemon vs compositor bindings cannot reliably emit hold events in all Wayland environments.
- How to obtain cursor position in Wayland for overlay placement (compositor-specific or portal limitations).
- How selection capture occurs relative to hotkey hold (capture first, then menu; or menu chooses command, then capture).
- Accessibility/keyboard-only equivalent.
- Timing precision and accidental activation prevention.

## Decisions

- Hold detection: compositor press/release bindings invoking `pixelens hotkey press <id>` and `pixelens hotkey release <id>`.
- Flow: menu first, then capture (command chosen before region selection).
- Cancel: release in center cancels.
- Sectors: dynamic from config/tags, not hardcoded list.
- Threshold default: 1.5s, configurable.

## Feasibility Notes

- Exact popup placement at cursor in Wayland is not universally available through standard protocols. Likely requires compositor-specific support or an overlay helper.
- Because menu-first flow captures after selection, the overlay can be anchored and then capture can be invoked from the chosen command.
- Config/tag schema must be defined before sector rendering.
