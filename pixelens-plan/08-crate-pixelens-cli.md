# 08 — Crate: pixelens-cli  ⚠️ BROKEN — FIX FIRST

**Role**: thin CLI client (`pixelens`). Talks to the daemon over IPC.
**Path**: `pixelens-cli/src/main.rs`
**Depends on**: `pixelens-ipc`, `pixelens-core`

## CURRENT STATE: ❌ DOES NOT COMPILE

The committed file was overwritten in commit `b6d5d33` ("Implement pixelens grab
workflow with error handling") with a hand-rolled stub that replaced the
previously-working IPC client. It does NOT build. Confirmed errors from
`cargo build`:

- `use clap::Parser;` but no `cli` module actually defined / no `clap` usage —
  and the code references `HELP`, `BINARY`, `VERSION`, `ExitCode` which are
  never declared.
- `wayland::is_wayland()` — **no such module or function exists** anywhere in
  the workspace. The real detector is `pixelens_capture::detect_display_server()`.
- `child.stdout.as_ref().unwrap().read_to_string(...)` — `ChildStdout` has no
  `read_to_string`; needs `read_to_end` / `read_to_string` on a `String`.
- `child.stdout.as_ref().unwrap().to_string()` — `ChildStdout` does not impl
  `Display`/`ToString`.
- The trailing `// ... rest of match arms` is a dangling comment — the original
  `match` is never closed, so the file is syntactically incomplete.

The build is **RED** because of this file. Nothing else in the workspace is the
cause.

## WHAT IT SHOULD BE (from commit `6dfb60a`, the real implementation)

Before the regression, the CLI had:
- `CliError` enum (`DaemonNotRunning`, `Ipc`, `Frame`, `Io`).
- `run_grab()` that connected to the daemon socket via `UnixStream`, sent an
  `IpcRequest { command: Grab, request_id: Uuid }` using `write_frame`, read
  the `IpcResponse` with `read_response`, and printed a human-readable summary
  (path / "No text found" / cancelled / daemon-not-running) — exiting non-zero
  on failure so shell pipelines detect it.
- `run_status()` similarly over IPC.
- A `HELP` constant matching the PRD CLI spec, and reserved-command handling
  (`search`, `ai`, `translate`, `image` → clear "not yet implemented" message).

## FIX PLAN (do this to restore green build)

1. Restore the IPC-based CLI from commit `6dfb60a`:
   `git show 6dfb60a:pixelens-cli/src/main.rs > pixelens-cli/src/main.rs`
   (verify it compiles; if `6dfb60a` is also not perfect, repair minimally).
2. Confirm `pixelens-cli/Cargo.toml` depends on `pixelens-ipc` and `clap`.
3. `cargo build` must pass for the whole workspace.
4. Do NOT re-add the `wayland::is_wayland()` / direct `slurp`/`grim` calls — the
   CLI must go through the daemon over IPC (PRD: "The CLI communicates
   exclusively through IPC. No DBus in v1."). Display detection is the
   daemon's job, not the CLI's.
5. Commit with a single, descriptive message, e.g.
   `fix(cli): restore IPC-based grab/status client (revert broken stub)`.

## Why this matters
The CLI is the user-facing entry point and the only thing the PRD's success
criteria (hotkey → select → text copied) actually touches end-to-end. A broken
CLI means the project cannot be demoed or tested by a human. This is the #1
blocker and takes priority over all new feature work.
