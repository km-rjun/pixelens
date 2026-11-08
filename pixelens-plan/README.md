# Pixelens Plan

This folder is the single source of truth for *where the project stands, what has
changed, and where it is going*. It is written for an AI agent (or a human) to
read on each session start so work can continue without re-deriving context.

The canonical requirement is the PRD at `/root/pixelens-v1.md` (647 lines). The
living docs under `/root/pixelens/docs/` (architecture.md, milestones.md) and
`/root/pixelens/tests/README.md` are part of the repo and complement this folder.

## Contents

| File | Purpose |
|---|---|
| [00-overview.md](00-overview.md) | What Pixelens is, in one page. |
| [01-goals.md](01-goals.md) | Absolute goals and success criteria (non-negotiable). |
| [02-architecture.md](02-architecture.md) | Component map and how they talk to each other. |
| [03-milestones.md](03-milestones.md) | M1–M11, status per milestone. |
| [04-crate-pixelens-core.md](04-crate-pixelens-core.md) | Shared types, errors, traits. |
| [05-crate-pixelens-ipc.md](05-crate-pixelens-ipc.md) | Wire protocol + socket framing. |
| [06-crate-pixelens-capture.md](06-crate-pixelens-capture.md) | Display detection + slurp/grim pipeline. |
| [07-crate-pixelens-daemon.md](07-crate-pixelens-daemon.md) | Daemon bin + lib, dispatch, IPC server. |
| [08-crate-pixelens-cli.md](08-crate-pixelens-cli.md) | CLI client. **Currently broken — see this file.** |
| [09-crate-stubs.md](09-crate-stubs.md) | ocr / overlay / notify / config status (mostly stubbed). |
| [10-progress.md](10-progress.md) | Live progress + build/git status. Read this first each session. |
| [11-changelog.md](11-changelog.md) | Chronological record of meaningful changes. |
| [12-roadmap.md](12-roadmap.md) | Where the project is headed + absolute end goals. |

## How to use this folder

1. On session start: read `10-progress.md`, then the element file for whatever
   you are about to touch.
2. When you make a change: append to `11-changelog.md` and update the relevant
   `03-milestones.md` / `10-progress.md` checkboxes.
3. Never mark a milestone done in here unless the build is green and the
   corresponding tests pass. A commit is not proof of correctness.

> Source of truth precedence: PRD (`/root/pixelens-v1.md`) > repo `docs/` > this
> folder. If this folder disagrees with the PRD, the PRD wins and this folder
> must be corrected.
