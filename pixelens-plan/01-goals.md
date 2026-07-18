# 01 — Absolute Goals

These are non-negotiable. They come from the PRD's Vision, Core Principle, and
Success Criteria. Every design decision is subordinate to these.

## The one thing that must work

> **A user can press a hotkey, select a region, and have the extracted text
> copied to their clipboard — in under 2 seconds on typical hardware — without
> menus, confirmations, AI prompts, account creation, cloud services, or
> provider selection.**

That is the entire reason Pixelens exists. Everything else is secondary.

## Core principle (hard constraint)

The fastest workflow is `Hotkey → Select Region → Text Copied`. The user must
never be forced through:
- menus or confirmations
- AI prompts
- account creation
- cloud services
- provider selection

`show_preview` defaults to `false`. The confirmation step in the capture flow
is opt-in only. The zero-friction path requires zero configuration.

## Success criteria (verbatim from PRD)

1. Press a hotkey.
2. Select a region.
3. Have text copied to the clipboard.
4. In under 2 seconds on typical hardware.

## Hard performance targets (PRD §Performance)

| Event | Target |
|---|---|
| Overlay appears after hotkey | < 100 ms |
| Capture response (region → extraction) | < 200 ms |
| OCR result (p95) | < 1 second |
| Daemon startup | < 1 second |
| Daemon idle memory | minimal, no leaks over long uptime |

Cold startup (first launch, Tesseract not yet warm) may exceed the OCR target;
document this in help. Subsequent captures must meet it.

## Behavioural guarantees

- **Clipboard default**: extract → copy. No confirmation. If extraction
  returns empty, do NOT write to clipboard; show `No text found in selection.`
- **Notifications**: all via libnotify/portal, all auto-dismiss, no modal
  dialogs, ever.
- **No DBus in v1.** CLI talks to daemon exclusively over the Unix socket.
- **Tesseract**: validated at startup, initialised once and kept warm. Missing
  Tesseract = clear install instructions + non-zero exit.
- **Display detection first**: happens at daemon startup before any other
  subsystem. Unknown environment = fatal error. No component may branch on
  display server type independently; they receive the mode from the detector.

## Definition of done for v1.0

All 11 milestones in `03-milestones.md` complete, integration tests pass on
both Wayland and X11 paths where applicable, performance benchmarks meet the
table above, documentation exists, and a v1.0 release is tagged.

## What "good" looks like for the agent

- Small, reviewable, single-logical-change commits (PRD §Git Workflow).
- Build is green (`cargo build`) and `cargo test` passes before declaring a
  milestone done.
- This plan folder is updated in lockstep with code changes.
- No introduction of abstractions for hypothetical future needs.
