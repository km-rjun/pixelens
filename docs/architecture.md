# Pixelens — Architecture

This directory holds design notes and decisions that don't fit in the
top-level `README.md` or the source code. Documents here are versioned
alongside the code.

| File | Topic |
|---|---|
| [PRD v1.0](../../pixelens-v1.md) | Source of truth for v1 requirements |
| [milestones.md](milestones.md) | Milestone-by-milestone breakdown |
| (more to follow) | — |

## Top-level architecture

```
                ┌──────────────────────────────┐
   hotkey       │        pixelensd            │
  ─────────►    │  ┌────────────────────────┐  │
                │  │  display-server det.   │  │   ┌──────────────┐
                │  └─────────┬──────────────┘  │   │ pixelens     │
                │            ▼                 │   │ (CLI client) │
                │  ┌────────────────────────┐  │   └──────┬───────┘
                │  │   CaptureProvider      │  │          │ IPC
                │  │  (Wayland | X11)       │  │          │ (Unix sock)
                │  └─────────┬──────────────┘  │          │
                │            ▼                 │          ▼
                │  ┌────────────────────────┐  │
                │  │   Selection overlay    │  │
                │  └─────────┬──────────────┘  │
                │            ▼                 │
                │  ┌────────────────────────┐  │
                │  │   OcrEngine (warm)     │  │
                │  └─────────┬──────────────┘  │
                │            ▼                 │
                │  ┌────────────────────────┐  │
                │  │   clipboard + notify   │  │
                │  └────────────────────────┘  │
                └──────────────────────────────┘
```

## Workspace crate map

See `/root/pixelens/Cargo.toml` and the PRD §"Repository Structure".

## IPC

Length-prefixed JSON over a Unix domain socket at
`$XDG_RUNTIME_DIR/pixelens.sock`. Every request carries a UUIDv4
`request_id`; responses echo it so `cancel` can be matched to the
in-flight session.
