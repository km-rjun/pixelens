# Pixelens Internal Documentation

These documents are maintainer-facing reference material. They are intentionally kept under `.internal/`, which is gitignored and is not part of the source tree shipped to users.

## Source of Truth

- `AGENTS.md` — project identity, command semantics, development rules
- `docs/PROJECT_STATE.md` — current status and known limitations
- `docs/ROADMAP.md` — planned work
- `README.md` — user-facing installation and usage
- `.internal/AUDIT.md` — latest code-vs-intent audit and open defects

## Index

| File | Purpose |
|------|---------|
| `ARCHITECTURE.md` | Runtime architecture, data flow, IPC, daemon lifecycle, capture/OCR/AI/action pipelines |
| `MODULE_MAP.md` | Crate and module ownership map with responsibilities |
| `COMMANDS.md` | CLI commands, options, semantics, exit codes, daemon service control |
| `CONFIG.md` | Configuration file, model settings, environment overrides, paths, tools |
| `DEVELOPMENT.md` | Build/test/lint workflows, feature flags, installation, packaging, contribution rules |
| `CHANGE_LOG.md` | Documentation baseline and milestone tracking |
| `AUDIT.md` | Defect inventory and recommended follow-up tests |

## Workspace Baseline

- Workspace version: `0.1.0` (`Cargo.toml`)
- Rust edition: `2021`, MSRV `1.77.2`
- Baseline git commit: `012646a`
- Baseline verification recorded in `.internal/CHANGE_LOG.md`

## Reading Order for New Maintainers

1. `AGENTS.md`
2. `../docs/PROJECT_STATE.md`
3. `ARCHITECTURE.md`
4. `MODULE_MAP.md`
5. `COMMANDS.md` and `CONFIG.md`
6. `AUDIT.md` before changing behavior
