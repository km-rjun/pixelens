# Architecture

## Crate Responsibilities

- `pixelens` — CLI entrypoint and daemon controller. Parses clap subcommands, performs IPC client calls, runs built-in GTK action bar.
- `pixelensd` — daemon executable. Creates `IpcServer`, handles Ctrl+C and shutdown notification, runs server loop.
- `pixelens-core` — shared library. Capture, OCR, AI, actions, configuration, IPC, menu, check, upload, search, hotkey, types, errors.

## Runtime Flow

1. CLI command obtains its content by screen selection via daemon `/grab`, except `ai` currently bypasses `do_grab` in the present code and sends prompt directly.
2. Daemon selects region with `slurp` (Wayland) or `slop` (X11), captures with `grim` or `scrot`.
3. OCR is performed by Tesseract using configured language.
4. For action commands, daemon dispatches `ActionHandler::execute`.
5. CLI consumes `Response` and prints results (URL, AI text, status).

## Wayland Capture Pipeline

- `WaylandCapture::select_region`: runs `slurp -f` or plain `slurp`; parses `x,y WxH`.
- `capture`: runs `grim -g <geometry> -`; writes PNG bytes to a unique `pixelens_<timestamp>.png` under OS temp dir.
- Wayland tools are the advertised/most tested backend.

## X11 Capture Pipeline

- `X11Capture` uses `slop -f "%x %y %w %h"` and `scrot -a geometry -o /tmp/pixelens_x11.png`.
- Code exists and is selectable when Wayland tools are missing, but README/Roadmap mark X11 unsupported/exploratory. Treat as experimental.

## OCR

- `create_engine()` requires `tesseract`.
- `perform_ocr(image_path, language)`.
- `clean_ocr_output` normalizes CRLF and trims blank boundaries in two variants: `ocr/mod.rs` (trait utility) and server-local `clean_ocr_output` (collapse multiple blank lines).

## AI

- OpenAI-compatible `/chat/completions` via blocking `ureq`.
- `OpenAiClient::build_request` sends text content always; attaches `image_url` data URI when `image_path` is provided and readable.
- `model_supports_vision` substring-matches configured model against `VISION_MODELS`.
- Retries temporary 429s with exponential backoff up to `MAX_RETRIES`; quota exhausted and auth errors return immediately.
- Vision mismatch returns `RequestFailed` if image provided for non-vision model.

## Actions

- `CopyHandler`: runs `wl-copy -- <text>` and returns text.
- `SearchHandler`: builds Google search URL; launches via `xdg-open`; currently returns empty string (`P1` defect; expected URL).
- `TranslateHandler`: returns prompt template, not completed translation.
- `ReverseImageHandler`: saves image to cache, optionally copies with `wl-copy --type image/png <path>`, opens browser to Google Lens or custom uploader flow.

## IPC

- Unix domain socket at `${runtime_dir}/pixelens/pixelens.sock` (`/tmp/pixelens/pixelens.sock` fallback).
- Protocol is newline-delimited JSON (`Request`/`Response`).
- `IpcServer::start` installs socket, accepts connections, spawns task per connection.
- `handle_request` dispatches ping, status, stop, tools, config, grab, OCR, AI, actions.
- Atomic `CAPTURING` guard prevents concurrent grabs.
- Stop returns `Stopped` but daemon process exits only through Ctrl+C path in current code (`P0` defect).

## Menu and Action Bar

- `grab` calls `show_action_bar`, a blocking GTK layer-shell window with buttons C/S/A/T/Esc.
- Menu choices are plain enum variants, no OCR context passed to backend.
- Stdout-based external backends (`fuzzel`, `wofi`, `stdin`) and `action_bar` construction remain available via `create_backend`, although `grab` calls `show_action_bar` directly.

## Upload and Reverse Image

- Providers: `0x0` and `custom` endpoint.
- Cache dir `~/.cache/pixelens`.
- Reverse image saves captured image as timestamped PNG in cache and opens Google Lens upload URL.
- `reverse_image_provider` config exists but `ReverseImageHandler` uses `image_upload_provider`/`image_upload_url`; `create_search_provider` supports `google_lens`.

## Configuration and Paths

- Config: `${config_dir}/pixelens/config.json`.
- Cache: `${cache_dir}/pixelens`.
- Runtime socket: `${runtime_dir}/pixelens/pixelens.sock`.
- Environment overrides accepted at load time.

## Error Model

- `PixelensError`, `CaptureError`, `OcrError`, `AiError`, `IpcError`.
- AI errors distinguish missing API key, 401, 429 temporary, quota exhausted.
- Capture cancellation represented as `CaptureError::RegionCancelled`.
