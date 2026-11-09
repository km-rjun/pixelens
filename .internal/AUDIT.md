# Pixelens Code-vs-Intent Audit

Generated from repository at `/home/rjun/projects/pixelens` (branch `main`, up to date with `origin/main`).

## Scope

This audit compares implementation against:

- `AGENTS.md`
- `docs/PROJECT_STATE.md`
- `docs/ROADMAP.md`
- `README.md`
- command help text and tests
- current Rust workspace implementation

No source changes were made while producing this audit.

## Baseline Verification

Commands run:

- `cargo fmt --all -- --check` — passed
- `cargo clippy --workspace --all-targets -- -D warnings` — passed
- `cargo build --workspace` — passed
- `cargo test --workspace` — passed (`135 passed; 0 failed; 5 ignored`)

Ignored tests include environment/container-only smoke tests (`wl-copy`, `tesseract`, Wayland/X11 tools, provider network).

## Summary

Status update (2026-10-20): the P0 items in this audit were implemented in the working tree. Remaining items are P1/P2 unless rechecked.

The Rust workspace builds, formats, passes clippy, and tests pass. Main defects are behavioral/intent mismatches rather than compilation failures:

- `pixelens ai` violates the selection-first command model and never captures/sends a screenshot.
- `pixelens daemon stop` can leave `pixelensd` running.
- The `grab` action bar discards the captured image path, so AI/image-aware actions cannot use vision.
- Text search does not return its search URL, despite callers expecting one.
- X11 capture is implemented and advertised as available even though documentation explicitly says X11 is not implemented/tested.

## P0 — Must Fix

### 1. `pixelens ai` does not select screen content
**Files/lines:** `crates/pixelens/src/main.rs:97`, `crates/pixelens/src/main.rs:304-320`, `crates/pixelens/src/main.rs:121-146`

**Evidence:** `Commands::Ai { prompt }` dispatches `cmd_ai(&prompt.unwrap_or_default(), None)` directly. `cmd_ai` never calls `do_grab()` and therefore sends an empty OCR text and `image_path: None` to the daemon.

**Expected:** `AGENTS.md` and `docs/PROJECT_STATE.md` define `pixelens ai` as selection-first: select region, OCR, send text + image to AI.

**Suggested fix:** Make `cmd_ai` call `do_grab()`, pass the OCR text as the prompt context, and pass the captured image path into `client.ai(..., Some(image_path))`. Add the prompt as an optional modifier rather than replacing the screen selection.

### 2. `pixelensd` does not stop from `pixelens daemon stop`
**Files/lines:** `crates/pixelens-core/src/ipc/server.rs:119-122`, `crates/pixelens-core/src/ipc/server.rs:67-69`, `crates/pixelensd/src/main.rs`

**Evidence:** `handle_request(Request::Stop)` returns `Response::Stopped`, and `stop()` only removes the socket file. The daemon task in `pixelensd::main` only exits via Ctrl+C; there is no shutdown propagation from the IPC handler.

**Expected:** `pixelens daemon stop` should perform graceful daemon shutdown.

**Impact:** The CLI reports “Daemon stopped” because the socket disappears, while the daemon process can remain orphaned/invisible to future status checks.

**Suggested fix:** Add a shutdown notify/channel to `IpcServer` and set it when `Stop` is received; either have `handle_request` notify the daemon process or handle stop directly with `std::process::exit` after responding. Preferred to use the existing `Notify` cleanly from the service.

### 3. `/grab` AI path discards captured image path
**Files/lines:** `crates/pixelens/src/main.rs:121-129`, `crates/pixelens/src/main.rs:148-196`

**Evidence:** `do_grab()` matches `Ok((_image_path, text, _ai_response))`, ignoring the image path returned by `client.grab(...)`. `cmd_grab` then dispatches `MenuChoice::Ai` as `Request::Ai { prompt, image_path: None }`.

**Expected:** `docs/PROJECT_STATE.md` says AI is text + image if the model supports vision; `ROADMAP.md` tracks verify image input.

**Suggested fix:** Change `do_grab()` to return both OCR text and image path, e.g. `Result<(String, String), i32>`, and pass that image path into AI requests from both the menu and `cmd_ai`.

## P1 — Important Behavioral Mismatches

### 4. Search action does not return its URL
**Files/lines:** `crates/pixelens-core/src/actions/search.rs:51-58`, consumers `crates/pixelens/src/main.rs:183-193`, `285-302`

**Evidence:** `SearchHandler::execute` opens the URL with `xdg-open`, but returns `Ok(String::new())`. CLI code prints the response as if it were a URL, producing blank output.

**Expected:** `docs/PROJECT_STATE.md` states `pixelens search` returns a search URL and opens the browser. `README.md` usage/examples imply obtaining a value.

**Suggested fix:** Return the constructed Google search URL from `execute`. Keep tests updated to assert it contains `google.com`.

### 5. X11 is implemented/advertising despite being unsupported
**Files/lines:** `crates/pixelens-core/src/capture/mod.rs:13-51`, `crates/pixelens-core/src/capture/x11.rs`, `README.md` status table and non-goal section

**Evidence:** `detect_backend` enables `X11Capture` when `slop` + `scrot` exist; `check_tools` reports them as X11 requirements. README says “X11 support | Not implemented”; ROADMAP lists X11 as later/exploratory. CI does not install X11 tools and does not exercise this path.

**Expected:** Either no X11 support is exposed, or documentation clearly states tested support.

**Suggested fix:** Gate/conservatively disable the X11 backend under a “tested/prototype” marker, or document it as experimental only; add tests before claiming support per AGENTS.md Development Rules.

### 6. AI prompt context in `grab` is hardcoded and loses user prompt
**Files/lines:** `crates/pixelens/src/main.rs:198`, `crates/pixelens/src/main.rs:228-233`

**Evidence:** `MenuChoice::Ai` constructs `"Describe or explain the following text:\n\n{}`"; `MenuChoice::Translate` hardcodes English instead of preserving any target language.

**Expected:** The CLI has `--prompt` only for direct `ai`, and `translate --to`, but the interactive menu lacks those contexts; project state seeks improved prompt construction.

**Suggested fix:** Centralize prompt construction in one helper; let the menu ask for confirmation/target or use default parameter.

## P2 — Quality/Robustness/Documentation Gaps

### 7. `pixelens check` has a duplicated `pixelensd` check and omitted checks
**File/lines:** `crates/pixelens-core/src/check/mod.rs:229-245`

**Evidence:** `run_check()` pushes `check_tool("pixelensd")` and then checks `grim`, `slurp`, `pixelensd` again. It does not explicitly check `tesseract`, `fuzzel`/`wofi`, or `scrot`/`slop` even though README lists them as prerequisites.

**Suggested fix:** Replace duplicate entry with distinct required tools (`pixelensd`, `tesseract`, `wl-copy`, `xdg-open`, `grim`, `slurp`, optional menu backends).

### 8. X11 capture uses fixed insecure `/tmp` path and can collide
**File/lines:** `crates/pixelens-core/src/capture/x11.rs:33-57`

**Evidence:** Output is always `/tmp/pixelens_x11.png`, then read by path. Multiple concurrent captures can overwrite each other.

**Suggested fix:** Write a unique temp file under the cache dir, as Wayland does.

### 9. Relative `wl-copy` behavior may not consume image files correctly
**File/lines:** `crates/pixelens-core/src/actions/reverse_image.rs:17-29`

**Evidence:** `copy_image` calls `wl-copy --type image/png <path>` as a path argument. `wl-copy` normally reads stdin; passing a filename may not upload image contents (tool behavior/environment dependent).

**Suggested fix:** Open the file and pipe bytes to `wl-copy` stdin, or validate with integration tests.

### 10. `TranslateHandler::execute` only builds a prompt
**File/lines:** `crates/pixelens-core/src/actions/translate.rs:9-16`

**Evidence:** Handler returns the prompt itself instead of a completed translation (main commands correctly use daemon AI). External menu and API action path only get a prompt.

**Expected:** The CLI command works, but menu-driven translate action quality is inconsistent.

**Suggested fix:** Either route translate via daemon, or document that handler returns a prompt template requiring AI invocation.

### 11. Systemd unit install defaults and assumptions
**Files/lines:** `packaging/systemd/pixelensd.service`, `scripts/install.sh:20-40`

**Evidence:** Unit is `WantedBy=default.target`, while README documents/recommends `systemctl --user enable --now pixelensd` and service after graphical session. `install.sh` does not copy service file; only mentions manual copy.

**Suggested fix:** Align `WantedBy` to documented target (`graphical-session.target`) or document why default target is used, and optionally install the service in script.

### 12. Daemon concurrency guard is not released on request errors
**File/lines:** `crates/pixelens-core/src/ipc/server.rs:139-148`

**Evidence:** `CAPTURING` is reset after `handle_grab` returns. On request parsing or handler-level logic before this wrapper, the guard is managed, but not if an unexpected panic occurs in `handle_grab`. Not a normal issue, but relevant for crash recovery.

**Suggested fix:** Use a RAII guard or catch panics around handle.

## Ambiguities Needing Decision

- **AI image input status:** `ROADMAP.md` lists “Ask AI with screenshot input (partial)”, while `docs/PROJECT_STATE.md` claims it is working. The implementation base64-encodes an image and attaches it, but command plumbing discards image paths, so actual end-to-end support is partial in `grab`/`ai`.
- **X11 support:** README says unsupported, but implementation advertises it. Need decide whether X11 is a supported platform or prototype code to remove/document.
- **`reverse_image_provider`:** Config has `reverse_image_provider`, but `pixelens image`/`ReverseImageHandler` does not use custom providers from this field; it relies on `image_upload_provider`/`image_upload_url`. README documents `reverse_image_provider` as `google_lens`, so behavior is slightly inconsistent but not broken.
- **Menu backend behavior:** `menu::detect_backend()` calls `action_bar::create_backend()` and starts a GTK thread even when no interactive menu is requested; callers often bypass it and call `show_action_bar()` synchronously.

## Recommended Next Audit Tests

- Add daemon-backed integration smoke test (start daemon, `pixelens daemon status`, `pixelens daemon stop`, verify process exits).
- Add tests around `pixelens ai` capturing and forwarding image path.
- Add handler test asserting Search returns expected URL.
- Add X11 behavior tests or remove X11 support.
- Add user documentation consistency for `check` tools.
