# 00 — Overview

**Pixelens** is a Linux-native utility that extracts text from anywhere on the
screen with minimal friction. The fast path is:

```
Hotkey → Select Region → Text Copied
```

It is written in Rust as a Cargo workspace of 9 crates. A background daemon
(`pixelensd`) owns the capture/OCR subsystems; a thin CLI (`pixelens`) talks to
it over a Unix domain socket.

## What it is NOT (PRD Non-Goals, v1.0)

Out of scope and must not influence architecture: AI integration, search
providers, translation, cloud OCR, user accounts, sync, history, plugins,
DBus, automation, circle/lasso selection, image search, OCR language
selection.

## Shapes of the thing

- **Daemon (`pixelensd`)**: detects display server, validates deps, keeps OCR
  warm, serves IPC. Built from `pixelens-daemon` (bin) + `pixelens_daemon`
  (lib) so integration tests can drive it in-process.
- **CLI (`pixelens`)**: shell-friendly client. `grab`/`copy` are the primary
  commands; `status`/`stop`/`config`/`version`/`help` exist too.
- **Crates** (see individual `04-*`/`05-*`/... files):
  `core`, `ipc`, `capture`, `ocr`, `overlay`, `notify`, `config`, `daemon`,
  `cli`.

## Current reality (as of writing)

The daemon, IPC, capture pipeline, and display detection are real and reasonably
complete for the v1-Wayland `slurp`+`grim` path. The **CLI is broken** — see
`08-crate-pixelens-cli.md`. The workspace does **not** build right now because of
that regression. OCR, overlay, notify, and config are still stubs.

> Do not trust "everything is done / pushed" claims from earlier sessions. The
> build is red until `08-crate-pixelens-cli.md` is resolved and `cargo build`
> passes.
