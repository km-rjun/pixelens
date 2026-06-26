# AGENTS.md — Pixelens

Read the PRD, `AGENTS.md`, and `docs/PROJECT_STATE.md` before changing code.

## Project Identity

Pixelens is a Linux-native, selection-first visual search tool. It is not a generic text-processing CLI.

The project is Rust-first. Do not introduce React, TypeScript, Vite, Tauri, or other frontend frameworks unless a real UI requirement has been evaluated and documented.

## Workspace

Current workspace contains only three crates:

- `pixelens` — CLI binary and daemon controller
- `pixelensd` — background daemon
- `pixelens-core` — capture, OCR, actions, configuration, IPC, and shared logic

Prefer focused Rust modules over creating additional crates.

## Daemon

`pixelensd` is the only daemon implementation. Never duplicate daemon or IPC server logic inside the CLI.

## Command Semantics

Main commands obtain their content from screen selection:

- `pixelens grab`
- `pixelens copy`
- `pixelens search`
- `pixelens ai`
- `pixelens translate`
- `pixelens image`

Do not redesign commands into generic positional text-input utilities.

Preserve the distinction:

- `grab` returns captured/OCR content without modifying the clipboard.
- `copy` copies captured OCR content.

## Development Rules

- Verify unfamiliar APIs and dependencies using official documentation.
- Do not invent platform support or claim untested functionality.
- Keep `main.rs` files small.
- Avoid duplicated pipelines and daemon implementations.
- Work in small logical commits.
- Run formatting, clippy, build, and tests.
- Push every completed commit.
- Do not report completion until pushed CI passes.
- Update `docs/PROJECT_STATE.md` after meaningful milestones, but keep it concise and current.
