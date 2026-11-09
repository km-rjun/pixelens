# Development

## Prerequisites for Contributors

Install Rust stable with clippy/rustfmt plus system dependencies used by CI:

```bash
sudo apt-get install -y libssl-dev tesseract-ocr tesseract-ocr-eng
```

Wayland integration needs `grim`, `slurp`, `wl-clipboard`, `xdg-open`; optional `fuzzel` or `wofi`. Layer-shell GTK build needs GTK3 and gtk-layer-shell when enabling `layer-shell`.

## Build

```bash
cargo build --workspace
cargo build --workspace --features layer-shell
```

Install script builds with layer-shell support:

```bash
./scripts/install.sh
```

## Format

```bash
cargo fmt --all -- --check
```

## Clippy

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

README documents `--all-features`, but CI removed it to avoid layer-shell system dependency.

## Tests

```bash
cargo test --workspace
```

Baseline: `135 passed; 0 failed; 5 ignored`. Ignored tests require local tools/session:
- `wl-copy`
- `tesseract`
- Wayland/X11 capture tools
- provider network

## Feature Flags

- `pixelens-core/layer-shell`: enables GTK3 + gtk-layer-shell action bar.
- `pixelens/layer-shell`: propagates to core.
- Default build disables graphical action bar and `show_action_bar` errors without it.

## Daemon and Service

- `pixelensd` is sole daemon implementation.
- `packaging/systemd/pixelensd.service` can be copied to `~/.config/systemd/user/`.
- `WantedBy=default.target`; service starts after `graphical-session.target`.

## Testing Design Notes

- `SearchHandler` and `ReverseImageHandler` are generic over launcher/clipboard/browser traits for unit testing.
- IPC server handler is a free async function; tests cover ping/status/stop/concurrency guard.
- Config env tests mutate process env; treat carefully in parallel suites.

## Commit and Push Rules (from AGENTS.md)

- Run formatting, clippy, build, and tests.
- Work in small logical commits.
- Push every completed commit.
- Do not report completion until pushed CI passes.
- Update `docs/PROJECT_STATE.md` after meaningful milestones, keeping it concise.

## Contribution Constraints

- Rust-first. Do not introduce React/TypeScript/Vite/Tauri/frontend frameworks unless documented.
- Keep `main.rs` small.
- Do not duplicate daemon or IPC server logic in CLI.
- Preserve command distinction: `grab` returns OCR content without clipboard mutation; `copy` copies captured OCR.
- Do not claim untested platform support.
- Update internal docs when architecture or command behavior changes.
