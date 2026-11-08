# Pixelens Unification — Strategy C Contract

**Date:** 2026-07-20
**Decision:** Option 3 + Strategy C (user: msg 532, 537, 548)
**Scope:** Merge the two divergent Pixelens efforts (features/core-loop + origin/main alt impl)
into one tree. features/core-loop = structural base; port main's richer capability modules.

---

## 1. Reality check (probed 2026-07-20)

- This VM: `10.0.0.55/24`, gateway `10.0.0.1`. Headless, no display server.
- `origin/main` (commit `012646af`) is NOT a Tauri GUI. It is an alternate Pixelens:
  nested `crates/{pixelens, pixelens-core, pixelensd}` workspace with AI (OpenAI-compatible +
  vision), search (google_lens), upload (custom), action-bar menu (gtk-layer-shell + fuzzel/wofi),
  reverse-image search, `pixelens check`. No global hotkeys, no Windows, no autostart, no portal.
- features/core-loop (our branch) is flat `pixelens-*` layout with: global hotkeys (keyhook),
  Windows backend (mock), autostart, portal-native capture (UM5, shipped), IPC daemon, config (TOML).
  NO AI/search/menu/action-bar/upload.
- **No merge base.** Integration is a PORT, not a git merge.

## 2. Config collision (resolved)

| Aspect        | features/core-loop (`pixelens-config`)        | origin/main (`pixelens-core::config`)   |
|---------------|------------------------------------------------|------------------------------------------|
| Format        | **TOML**, nested `[general]/[capture]/[ocr]`  | JSON, flat                               |
| Default model | — (no AI)                                      | `gpt-4o`                                 |
| OCR lang      | `ocr.engine = "tesseract"`                     | `ocr_language = "eng"`                   |
| Hotkey        | `general.hotkey = "Super+Shift+T"`             | `hotkey` (flat)                          |
| AI fields     | none                                           | `api_endpoint`, `api_key`, `model`, `menu_backend`, `image_upload_*`, `reverse_image_provider` |

**Resolution: standardize on TOML.** Our branch ships TOML config (tested, shipped). We add
`main`'s AI/search/upload fields as new nested sections. No JSON in the unified tree.

Final TOML shape:
```toml
[general]
autostart = false
theme = "dark"
hotkey = "Super+Shift+T"

[capture]
show_preview = true

[ocr]
engine = "tesseract"
language = "eng"          # migrated from main's ocr_language

[ai]
endpoint = "http://10.0.0.88:11434/v1"
api_key = ""              # Ollama needs none; OpenAI-compatible servers may
model = "hermes-qwen3"    # default Ollama model (text). Vision models: see allowlist
menu_backend = "fuzzel"   # fuzzel | wofi | stdin

[search]
provider = "google_lens"

[upload]
endpoint = ""             # custom upload backend (main: zeroxzero)
provider = ""

[reverse_image]
provider = "google_lens"
```

## 3. Target crate layout (Strategic C — modular, our skeleton)

Keep existing flat crates; ADD: `pixelens-ai`, `pixelens-search`, `pixelens-menu`.
Reconcile IPC so daemon handles AI.

```
pixelens-core       # traits, geometry, error, types (shared)        [KEEP]
pixelens-ipc        # Request/Response protocol + client/server      [EXTEND: add Ai/Search/Image/Translate arms]
pixelens-config     # TOML model + io                                [EXTEND: ai/search/upload/reverse_image sections]
pixelens-ocr        # OCR engine trait + tesseract                   [KEEP]
pixelens-capture    # capture providers (slurp/grim, portal)         [KEEP]
pixelens-portal     # xdg-desktop-portal backend                     [KEEP verbatim]
pixelens-overlay    # preview overlay                                [KEEP]
pixelens-notify     # desktop notifications                          [KEEP]
pixelens-keyhook    # global hotkeys (linux/win)                     [KEEP verbatim]
pixelens-ai         # NEW: OpenAI-compatible client, vision, retries [PORT from main ai/]
pixelens-search     # NEW: google_lens search + reverse-image URL    [PORT from main search/upload]
pixelens-menu       # NEW: action-bar menu (gtk-layer-shell+fuzzel)  [PORT from main menu/]
pixelens-daemon     # IPC server; dispatch to ocr/ai/search/upload   [EXTEND: AI arm]
pixelens-cli        # unified CLI subcommands                        [REWRITE: Grab/Copy/Search/Ai/Translate/Image/Check/Daemon/Config]
```

## 4. AI crate spec (pixelens-ai)

Port `main`'s `ai/mod.rs` OpenAiClient verbatim in logic. Changes:
- `ureq` (sync) → `reqwest` (async, tokio) OR keep `ureq` for drop-in. DECISION: use `reqwest`
  with a blocking wrapper to match our async daemon, OR expose `async fn chat`. Keep `build_request`
  / `parse_response` pure (fully unit-tested).
- Types `AiRequest { prompt, image_path: Option<String> }`, `AiResponse { content, model }`
  live in `pixelens-core::types` (shared with IPC).
- `model_supports_vision()` kept; default model for Ollama = `llava` (vision-capable).
- Retry/backoff (MAX_RETRIES=3, exp backoff + jitter) kept.
- `validate_api_key` relaxed: Ollama needs NO key, so treat empty key as OK when endpoint
  is Ollama (host contains `11434` or `ollama`) OR when `ai.require_key = false` config flag.
- **Tests:** all logic tests from main ported (text-only, image embed, missing-file fallback,
  non-vision rejection, parse valid/invalid, backoff bounds) — run against a LOCAL mock HTTP
  server (tiny axum/warp or a hand-rolled TcpListener returning canned JSON). These run GREEN here.
- **Live Ollama test:** `#[ignore] fn test_live_ollama_chat` (sync, reqwest::blocking)
  hitting `env::var("PIXELENS_LIVE_AI_ENDPOINT").unwrap_or("http://10.0.0.88:11434/v1")`
  with model `PIXELENS_LIVE_AI_MODEL` (default `hermes-qwen3:latest`). Run with
  `cargo test -p pixelens-ai -- --ignored` on a host that can reach Ollama.
- **Verified 2026-07-20:** `10.0.0.88:11434` IS REACHABLE from this VM (HTTP 200,
  models: hermes-qwen3, qwen3.5:9b, ornith). Live text-only chat returned `LIVE AI OK`
  — end-to-end proven against real hardware (commit `00f2e25`, pushed). Vision arm NOT
  yet verified: `model_supports_vision()` allowlist (`gpt-4o/gpt-4-turbo/claude-3-*/llava/
  bakllava`) lacks qwen models → screenshot→AI currently rejected, falls back to text.
  Follow-up: extend the allowlist (product decision).

## 5. IPC reconciliation (pixelens-ipc)

Adopt `main`'s protocol `Request`/`Response` enums (Ping/Status/Stop/CheckTools/GetConfig/
Grab/Ocr/Ai/Action) and `AiRequest`/`AiResponse`/`ActionType`. Our daemon already has IPC —
extend it so `Request::Ai { prompt, image_path }` dispatches to `pixelens-ai::OpenAiClient`
(sync→async bridge inside daemon task). `ActionType` covers CopyToClipboard / SearchWeb /
ReverseImageSearch / AskAi.

## 6. Daemon dispatch extension (pixelens-daemon)

Add dispatch arms:
- `Request::Ai` → load config, build `OpenAiClient`, call `chat`, return `Response::AiResult`.
- `Request::Ocr` already exists → feed OCR text into AI prompt for Translate/Copy flows.
- Search/ReverseImage → `pixelens-search` (build google_lens URL, open via xdg-open,
  return URL string).
- Upload → `pixelens-search`/`upload` (POST image to custom endpoint, return URL).

## 7. Menu (pixelens-menu)

Port `main`'s `menu/` (action_bar with gtk-layer-shell + fuzzel/wofi/stdin backends).
Headless VM: compile-only; `menu_backend = "stdin"` path is unit-testable (reads choice from
stdin). gtk/fuzzel paths compile but cannot run here (no display). Keep `MenuChoice` enum.

## 8. CLI unification (pixelens-cli)

Rewrite subcommands to: Grab, Copy, Search, Ai {--prompt}, Translate {--to}, Image, Check,
Daemon {start|status|stop}, Config {--endpoint|--model|--language|--hotkey}. Mirrors `main`'s
CLI shape; talks to daemon over IPC; falls back to direct capture when daemon absent (optional).

## 9. Next-phase upgrade (after unification merged)

UM6 / M3 and beyond (see 13-roadmap-upgrades.md) resume once the tree is unified and AI works.
Priority order post-unification:
1. ~~Live Ollama verify~~ ✅ DONE 2026-07-20 (text-only, `10.0.0.88`, `00f2e25`).
2. Vision allowlist extension (qwen models) — product decision, not yet done.
3. Windows backend real testing (cross-compile gate already in place).
4. Packaging / autostart polish.

## 10. Execution gating (per step)

- Each crate change: `cargo fmt --check` + `cargo clippy --workspace --all-targets -- -D warnings`
  + `cargo test -p <crate>` green before marking done.
- Disk mitigation: `cargo clean` before heavy builds (7.8G root, routinely full).
- No `origin/main` push, ever. New work on a branch off features/core-loop; push only on
  explicit go-ahead.
- Planning docs (this file + 10/11/13) are LOCAL-ONLY — never pushed.

## 11. Outstanding env blocks (honest)
- Ollama host `10.0.0.88:11434` IS reachable (2026-07-20 verified; HTTP 200, real models).
  Live TEXT-ONLY AI chat proven (`00f2e25`). Vision arm still gated: `model_supports_vision()`
  allowlist lacks qwen models → screenshot→AI currently rejected, falls back to text.
- Headless: no display — menu gtk/fuzzel, live capture, OCR cannot run here. Mocked/compiled only.
- Daemon win-msvc: pre-existing `tokio::net::UnixListener` E0432 on Windows target (not a
  regression). Windows runtime QA impossible here regardless.

## 12. Status as of 2026-07-20 (u1–u10 complete on features/core-loop)

The unification is **functionally closed** in code. What the contract in §1–§8
anticipated vs what shipped:

| Strategy-C item | State | Notes |
|---|---|---|
| `pixelens-ai` (port) | ✅ DONE | OpenAI-compatible client, vision, retries; `reqwest::blocking`. Tested vs mock HTTP; live text-only `#[ignore]` verified @ `10.0.0.88` (`00f2e25`); vision arm allowlist TBD. |
| `pixelens-search` (port) | ✅ DONE | google_lens URL build + reverse-image + upload (custom/zeroxzero). 21 tests. |
| `pixelens-menu` (port) | ✅ DONE (u7) | types/stdin/fuzzel/wofi/action_bar[menu-gtk]/factory. 21 tests. |
| Daemon AI dispatch arm | ✅ DONE | `handle_ai` wired to `pixelens-ai`; `AiPayload{prompt,image_path}`. |
| Daemon menu dispatch arm | ✅ DONE (u8) | `handle_grab` → `decide_action` → Copy/Search/Ai/Translate via existing handlers; Cancel/fallback → auto-copy. |
| CLI unified subcommands | 🟡 partial | Grab/Copy/Search/Ai/Translate/Config/Daemon/Autostart/Hotkey present; `Image`/`Check` shape matches `main` intent. CLI→daemon IPC + direct-capture fallback wired. |
| Config: ai/search/upload/reverse_image sections | ✅ DONE | nested TOML (no JSON), per §2 resolution. |
| v1 core loop (hotkey→capture→OCR→menu→action) | ✅ code-complete | all stages present + wired; headless VM blocks live end-to-end QA. |

**Remaining gaps before a v1 *release*** (not core-logic gaps):
1. Live Linux desktop QA — real hotkey capture, region select, OCR, clipboard.
   Headless VM cannot exercise this; needs a Wayland/X11 session.
2. Windows `cargo build --target x86_64-pc-windows-msvc` LINK — pre-existing
   tokio `UnixListener` E0432; only `cargo check` (type-check) passes. Not a
   regression; needs a real Windows link or a tokio feature gate.
3. Release artifact — deb/rpm/AppImage packaging not done.
4. Live AI verify — Ollama `10.0.0.1:11434` unreachable from VM; `#[ignore]` only.

Next upgrade phase (UM6 multi-display / M3 portal PipeWire / live-AI-verify)
resumes after v1 release-QA on a capable host.
