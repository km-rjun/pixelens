# Module Map

## Workspace Crates

| Crate | Files | Responsibility |
|-------|-------|----------------|
| `pixelens` | `crates/pixelens/src/main.rs` | CLI parse/dispatch, daemon start/status/stop, config display, action-bar flow |
| `pixelens` tests | `crates/pixelens/tests/cli_tests.rs` | CLI binary help/version/daemon status smoke tests |
| `pixelensd` | `crates/pixelensd/src/main.rs` | Daemon process lifecycle and signal handling |
| `pixelens-core` | `src/lib.rs` + modules | Shared implementation invoked by CLI and daemon |

## Core Modules

| Module | Path | Summary |
|--------|------|---------|
| `actions` | `src/actions/mod.rs` | `ActionHandler` trait, handler factory, action choices/menu helper |
| `actions/copy` | `src/actions/copy.rs` | Clipboard copy via `wl-copy` |
| `actions/search` | `src/actions/search.rs` | Google search URL construction and browser launch |
| `actions/translate` | `src/actions/translate.rs` | Prompt template generation only |
| `actions/reverse_image` | `src/actions/reverse_image.rs` | Cache save, clipboard copy, browser launch, optional upload |
| `ai` | `src/ai/mod.rs` | OpenAI-compatible chat client, vision model detection, retries |
| `capture` | `src/capture/mod.rs` | `ScreenCapture` trait, backend detection, tool checks |
| `capture/wayland` | `src/capture/wayland.rs` | `slurp` + `grim` capture |
| `capture/x11` | `src/capture/x11.rs` | `slop` + `scrot` capture (experimental) |
| `check` | `src/check/mod.rs` | Environment/config diagnostic items |
| `config` | `src/config/mod.rs` | `Config` load/save/env override |
| `error` | `src/error.rs` | Error enums and rate-limit kind |
| `hotkey` | `src/hotkey/mod.rs`, `parser.rs` | Hotkey parse/match utility |
| `ipc` | `src/ipc/protocol.rs` | Request/Response IPC schema |
| `ipc/client` | `src/ipc/client.rs` | Unix socket client |
| `ipc/server` | `src/ipc/server.rs` | Unix socket server and request handlers |
| `menu` | `src/menu/mod.rs` | `MenuChoice`, backend factory, traits |
| `menu/action_bar` | `src/menu/action_bar.rs` | GTK layer-shell action bar and fallback backend |
| `menu/fuzzel` | `src/menu/fuzzel.rs` | External fuzzel backend |
| `menu/wofi` | `src/menu/wofi.rs` | External wofi backend |
| `menu/stdin` | `src/menu/stdin.rs` | Terminal fallback backend |
| `ocr` | `src/ocr/mod.rs` | OCR trait, engine factory, tool checks, text cleanup |
| `ocr/tesseract` | `src/ocr/tesseract.rs` | Tesseract process invocation |
| `search` | `src/search/mod.rs`, `google_lens.rs` | Reverse search provider trait/Google Lens |
| `types` | `src/types.rs` | Shared serializable domain types and action enum |
| `upload` | `src/upload/mod.rs`, `custom.rs`, `zeroxzero.rs` | Image uploader factory and providers |

## Key Shared Types

- `CaptureRegion`, `CaptureResult`, `OcrResult`, `AiRequest`, `AiResponse`, `ActionPayload`, `ActionType`
- `MenuChoice`
- `Request`, `Response`
- `Config`

## IPC Request/Response

- Request variants: `Ping`, `Status`, `Stop`, `CheckTools`, `GetConfig`, `Grab { search, ai }`, `Ocr { image_path, language }`, `Ai { prompt, image_path }`, `Action { action, text, image_path }`.
- Response variants: `Pong`, `Stopped`, `Status`, `ToolsStatus`, `Config`, `GrabResult`, `CaptureResult`, `OcrResult`, `AiResult`, `ActionResult`, `Error`.

## Testing Ownership

- Unit tests live beside modules.
- CLI help/version/status integration in `crates/pixelens/tests/cli_tests.rs`.
- Environment-dependent tests are ignored (`wl-copy`, `tesseract`, capture tools, provider network).
