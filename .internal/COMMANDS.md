# Commands

## CLI Shape

Binary: `pixelens` (from crate `pixelens`). Uses clap derive. Global options depend on subcommand. Default feature set has no `layer-shell`; install builds with `--features layer-shell`.

### `pixelens grab`

Select a region, OCR it, then open built-in action bar. Choices: Copy, Search, Ask AI, Translate, Cancel.

- No args.
- Exit code: `0` normally, `1` on capture/daemon/menu error.
- Selection cancellation is reported as error.

### `pixelens copy`

Select region, OCR text, copy via daemon action handler to clipboard.

- No args.
- Prints no URL; success usually silent except errors.
- Exit `0` on success.

### `pixelens search`

Select region, OCR text, build Google search URL and open browser.

- No args.
- Prints returned URL.
- Known code issue: `SearchHandler::execute` currently returns empty string; audit flags expected URL.

### `pixelens ai [--prompt PROMPT]`

Direct AI command. In current code, `cmd_ai` does not select screen content; it calls daemon AI directly with prompt and optional image. Project state/audit expects selection-first behavior.

- `--prompt/-p`? Actually `--prompt` long only.
- `--image` not exposed; main passes `None`.
- Exit `0` on success, `1` otherwise.

### `pixelens translate [--to LANGUAGE]`

Select region, OCR text, send translation prompt to AI.

- `--to`, default `English`.
- Prints translation.

### `pixelens image`

Select region (daemon grab without OCR/AI use text ignored), save image, open Google Lens/upload page if configured.

- No args.
- Prints handler result.

### `pixelens check`

Runs environment/configuration diagnostics.

- No args.
- Exit `0` if no failures, `1` if any `Fail`.
- Output lines `[ok]`, `[warn]`, `[fail]`, then summary.

### `pixelens daemon start`

Start `pixelensd` if not already running. Looks for sibling `pixelensd` next to current executable.

- No args.
- Waits 500ms and pings.

### `pixelens daemon status`

Ping daemon via IPC; prints missing capture/OCR tools.

- No args.
- Returns success even if daemon stopped.

### `pixelens daemon stop`

Send `Request::Stop` over IPC.

- No args.
- Prints `Daemon stopped` or `Daemon was not running`.

### `pixelens config [OPTIONS]`

Display or update configuration.

Options:
- `--endpoint`
- `--model`
- `--language`
- `--hotkey`
- No args prints endpoint/model/OCR language/hotkey and masked API key.

### `pixelens version`

Print `pixelens <CARGO_PKG_VERSION>` (0.1.0 baseline).

## Daemon IPC Requests

Daemon controller/client methods:
- `ping`, `status`, `stop`, `grab(search, ai)`, `ocr(image_path, language)`, `ai(prompt, image_path)`, `action(ActionType, text, image_path)`.

## Exit Codes

- `0`: success / cancellation handled where applicable.
- `1`: invalid/failed operation, missing daemon, capture failure, menu error, check failures.
- Process exits immediately via `std::process::exit`.
