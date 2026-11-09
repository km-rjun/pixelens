# Configuration

## Config File Location

`$config_dir/pixelens/config.json`, typically `~/.config/pixelens/config.json`. Created by install script but may be absent; defaults are used.

## Config Fields

| Field | Type | Default | Meaning |
|-------|------|---------|---------|
| `api_endpoint` | string | `https://api.openai.com/v1` | OpenAI-compatible base URL |
| `api_key` | string | `` | API key; normally supplied via env |
| `model` | string | `gpt-4o` | Model name used for AI requests |
| `ocr_language` | string | `eng` | Tesseract language code |
| `hotkey` | string | `Ctrl+Shift+C` | Informational configured hotkey |
| `menu_backend` | string | `auto` | `auto`, `action_bar`, `fuzzel`, `wofi`, `stdin` |
| `image_upload_provider` | string | `` | `` (disabled), `0x0`, or `custom` |
| `image_upload_url` | string | `` | Endpoint required for `custom` |
| `reverse_image_provider` | string | `google_lens` | Configured but handler currently uses upload provider flow |

## Environment Variables

Read by `Config::load`:
- `PIXELENS_API_KEY` — overrides `api_key`.
- `PIXELENS_MODEL` — overrides `model`.
- `PIXELENS_API_ENDPOINT` — overrides `api_endpoint`.
- `PIXELENS_OCR_LANGUAGE` — overrides `ocr_language`.

`Config::save` clears `api_key` in file when `PIXELENS_API_KEY` env var is set, so env-provided key is not persisted.

## Paths

- Config: `${config_dir}/pixelens`
- Cache: `${cache_dir}/pixelens` (screenshots for reverse image)
- Runtime socket: `${runtime_dir}/pixelens/pixelens.sock` (fallback `/tmp/pixelens/pixelens.sock`)
- Capture images: OS temp dir (`/tmp/pixelens_<timestamp>.png` Wayland; `/tmp/pixelens_x11.png` X11)

## Tooling and Checks

`pixelens check` validates:
- `pixelensd`, `grim`, `slurp`, `pixelensd` (duplicate in current code), `wl-copy`, `xdg-open`
- Wayland session (`WAYLAND_DISPLAY`)
- daemon socket existence
- config/cache directories and writability
- config file readability
- API key set/missing
- configured model/endpoint
- reverse image provider
- Tesseract availability and configured language
- layer-shell build feature

Capture tool check (`check_tools`) reports missing `slurp`/`grim` unless either Wayland or X11 tool pairs are all present; upload/menu/prerequisites beyond hardcoded list are not fully checked.

## AI Vision

- `model_supports_vision` matches model substrings.
- If image path is attached for non-vision model, `build_request` returns error.
- Missing image file logs warning and falls back to text-only.

## Upload Providers

- `0x0`: `ZeroXZeroUploader`
- `custom`: `CustomUploader` using `image_upload_url`
- Empty provider disables automatic upload; reverse image opens Google Lens upload page and copies image to clipboard.
