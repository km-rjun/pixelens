//! Request dispatcher: maps a [`Command`] to its handler.
//!
//! v1 only wires `Grab` to the real capture pipeline. Other commands
//! return a clear "not yet implemented" error so the CLI sees a
//! structured failure rather than a timeout.

use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use pixelens_ai::{AiRequest, OpenAiClient};
use pixelens_capture::GrabOutcome;
use pixelens_core::{
    CaptureError, CaptureImage, CaptureProvider, CaptureRequest, OcrEngine, PixelensError, Rect,
};
use pixelens_ipc::{
    AiPayload, AiResponsePayload, CopyPayload, GrabResponsePayload, IpcRequest, IpcResponse,
    ResponseStatus, ReverseImagePayload, SearchPayload, SearchResponsePayload, SetPreviewPayload,
    TranslatePayload,
};
use pixelens_menu::{self, MenuBackend, MenuChoice};
use pixelens_search::{build_search_url, ReverseImageSearcher};

use crate::clipboard::copy_text;
use crate::state::DaemonState;
use pixelens_notify::{notify_success, NotificationKind};

/// Error message prefix for user-facing errors.
const USER_ERROR_PREFIX: &str = "error: ";

/// Build a user-friendly error response.
fn user_error(request_id: &str, msg: &str) -> IpcResponse {
    IpcResponse::error(request_id.to_string(), format!("{USER_ERROR_PREFIX}{msg}"))
        .with_status(ResponseStatus::Error)
}

/// Build a user-friendly error response with a hint.
#[allow(dead_code)]
fn user_error_with_hint(request_id: &str, msg: &str, hint: &str) -> IpcResponse {
    IpcResponse::error(
        request_id.to_string(),
        format!("{USER_ERROR_PREFIX}{msg}\n  hint: {hint}"),
    )
    .with_status(ResponseStatus::Error)
}

pub struct Dispatcher {
    state: Arc<DaemonState>,
}

impl Dispatcher {
    pub fn new(state: Arc<DaemonState>) -> Self {
        Self { state }
    }

    /// Dispatch a request and produce a response. Always returns a
    /// typed [`IpcResponse`] — protocol-level errors are reported as
    /// `ResponseStatus::Error` rather than propagated, so the client
    /// always gets a well-formed frame.
    pub async fn dispatch(&self, request: IpcRequest) -> IpcResponse {
        use pixelens_ipc::Command;

        match request.command {
            Command::Grab => self.handle_grab(&request).await,
            Command::Status => self.handle_status(&request),
            Command::Stop => self.handle_stop(&request),
            Command::ConfigGet => self.handle_not_implemented(&request, "config get"),
            Command::ConfigSet => self.handle_not_implemented(&request, "config set"),
            Command::Cancel => self.handle_not_implemented(&request, "cancel"),
            Command::Redetect => self.handle_redetect(&request),
            Command::SetPreview => self.handle_set_preview(&request),
            Command::Ai => self.handle_ai(&request).await,
            Command::Search => self.handle_search(&request),
            Command::ReverseImage => self.handle_reverse_image(&request).await,
            Command::Translate => self.handle_translate(&request).await,
            Command::Copy => self.handle_copy(&request),
        }
    }

    async fn handle_grab(&self, request: &IpcRequest) -> IpcResponse {
        // UM4: the effective preview flag is the one-shot override (if
        // set for *this* grab) falling back to `capture.show_preview`.
        // We consume the override so the next grab reverts to config.
        let effective_preview = self.state.preview_for_next_grab();
        let override_was_set = self.state.take_preview_override().is_some();
        if override_was_set {
            tracing::info!(
                effective_preview,
                "UM4 one-shot preview override applied for this grab"
            );
        } else {
            tracing::debug!(
                effective_preview,
                "preview resolved from config (no one-shot override)"
            );
        }

        // UM5: prefer the portal-native backend when present; otherwise
        // fall back to the universal slurp/grim pipeline. Both branches
        // resolve to the same locals so the shared tail below runs
        // identically for either capture source.
        //
        // `region` is the screen-space rectangle, `bytes` the captured
        // image size in bytes, `path` a stable identifier for the grab,
        // and `text` the OCR result (possibly empty).
        let (region, bytes, path, text): (Rect, u64, String, String) = if let Some(backend) =
            &self.state.portal_backend
        {
            // Portal fast-path. `capture()` returns in-memory pixels
            // (no temp PNG written); OCR runs directly on the image.
            let req = CaptureRequest {
                session_id: format!("grab-{}", now_ms()),
            };
            let raw = match tokio::task::block_in_place(|| backend.capture(&req)) {
                Ok(r) => r,
                Err(e) => {
                    // A user abort in the portal/slurp selector is
                    // surfaced as a `Selector` error — treat it as a
                    // cancel, not a failure.
                    if matches!(e, PixelensError::Capture(CaptureError::Selector(_))) {
                        return IpcResponse::cancelled(request.request_id.clone());
                    }
                    tracing::warn!(error = %e, "portal grab failed");
                    return IpcResponse::error(request.request_id.clone(), e.to_string());
                }
            };
            let region = raw.region;
            // When the portal backend wrote an on-disk capture (the
            // slurp/grim fallback, which is what runs headless), report that
            // real path so the grab contract matches v1 (a file exists and
            // OCR can run on it). Only when the backend returns in-memory
            // pixels with no file do we synthesize a `portal://` identifier.
            let (bytes, path, ocr_source) = match &raw.path {
                Some(p) => {
                    let bytes = std::fs::metadata(p)
                        .map(|m| m.len())
                        .unwrap_or_else(|_| raw.image.width as u64 * raw.image.height as u64 * 4);
                    (
                        bytes,
                        p.to_string_lossy().into_owned(),
                        self.ocr_text_path(p),
                    )
                }
                None => {
                    let bytes = (raw.image.width as u64) * (raw.image.height as u64) * 4;
                    let path = format!("portal://capture-{}", now_ms());
                    (bytes, path, self.ocr_text(&raw.image))
                }
            };
            let text = ocr_source;
            (region, bytes, path, text)
        } else {
            let Some(pipeline) = &self.state.pipeline else {
                return user_error(
                    &request.request_id,
                    "capture pipeline not initialised: install slurp and grim, then restart the daemon",
                );
            };
            // Run the pipeline on a blocking thread: slurp / grim do
            // real I/O and may block while the overlay is on screen.
            let result = tokio::task::block_in_place(|| pipeline.run());
            let outcome = match result {
                Ok(o) => o,
                Err(e) => {
                    tracing::warn!(error = %e, "grab failed");
                    return user_error(&request.request_id, &e.to_string());
                }
            };
            match outcome {
                GrabOutcome::Captured {
                    path,
                    region,
                    bytes,
                } => {
                    let text = self.ocr_text_path(&path);
                    (region, bytes, path.to_string_lossy().into_owned(), text)
                }
                GrabOutcome::Cancelled => {
                    return IpcResponse::cancelled(request.request_id.clone())
                }
            }
        };

        // M5: run OCR on the captured image and attach the text to the
        // response. OCR failure is non-fatal — the capture already
        // succeeded, so we return whatever text we got (possibly empty)
        // rather than failing the whole grab.
        // (OCR is computed inside the capture branch above; `text` is
        // already resolved here.)

        // M7: complete the core loop. The capture + OCR already succeeded
        // u8: rather than unconditionally copying, consult the action
        // menu and dispatch the user's chosen side effect (copy/search/ai/
        // translate). `Cancel` or a backend failure degrades gracefully to a
        // plain capture report — a menu hiccup never turns a good grab into a
        // failed one. The grab response below always reports the capture
        // metadata regardless of which action ran.
        let _action_resp = self.decide_action(&text, &request.request_id).await;

        let payload = GrabResponsePayload {
            path,
            region: region.into(),
            bytes,
            captured_at_ms: now_ms(),
            text,
        };
        match IpcResponse::ok(request.request_id.clone(), &payload) {
            Ok(resp) => resp,
            Err(e) => IpcResponse::error(
                request.request_id.clone(),
                format!("failed to serialise grab response: {e}"),
            ),
        }
    }

    /// After an OCR capture, consult the action menu and run the chosen command.
    ///
    /// The backend comes from `DaemonState::menu_override` when present (tests /
    /// embedders), otherwise auto-detected via [`pixelens_menu::detect_backend`].
    /// If no backend is usable or it errors, we *gracefully degrade* to the
    /// previous v1 behavior (copy the OCR text) — the menu is never a hard
    /// failure point. A `Cancel` choice short-circuits to a no-op so the grab
    /// still reports the capture.
    ///
    /// Returns the response produced by running the chosen action (or the
    /// fallback), so callers/tests can observe which side effect fired.
    async fn decide_action(&self, ocr_text: &str, request_id: &str) -> IpcResponse {
        let backend: std::sync::Arc<dyn MenuBackend + Send + Sync> = match self
            .state
            .menu_override
            .clone()
        {
            Some(b) => b,
            None => match pixelens_menu::detect_backend() {
                Ok(b) => std::sync::Arc::from(b),
                Err(e) => {
                    tracing::warn!(error = %e, "menu backend unavailable; copying to clipboard");
                    return self.auto_copy_fallback(ocr_text, request_id).await;
                }
            },
        };

        let choice = match backend.show_menu(ocr_text) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "menu backend failed; copying to clipboard");
                return self.auto_copy_fallback(ocr_text, request_id).await;
            }
        };

        match choice {
            MenuChoice::Cancel => {
                tracing::info!("menu choice: cancel — capture reported, no action");
                IpcResponse::ok(
                    request_id.to_string(),
                    AiResponsePayload {
                        text: "capture cancelled — no action".to_string(),
                        model: "menu".to_string(),
                    },
                )
                .expect("serialize cancel payload")
            }
            // Run the chosen action through its concrete handler. We call the
            // handler directly (not `dispatch`) to avoid a recursive `async fn`
            // future (E0733).
            other => self.run_menu_choice(other, ocr_text, request_id).await,
        }
    }

    /// Dispatch a `MenuChoice` to its concrete handler without recursing through
    /// `dispatch`. Builds the minimal `IpcRequest` each handler expects and calls
    /// the matching `handle_*` method directly.
    async fn run_menu_choice(
        &self,
        choice: MenuChoice,
        ocr_text: &str,
        request_id: &str,
    ) -> IpcResponse {
        let req = choice.to_request(ocr_text, format!("{request_id}-menu"));
        match choice {
            MenuChoice::Copy => self.handle_copy(&req),
            MenuChoice::Search => self.handle_search(&req),
            MenuChoice::Ai => self.handle_ai(&req).await,
            MenuChoice::Translate => self.handle_translate(&req).await,
            MenuChoice::Cancel => self.handle_copy(&req), // unreachable: handled above
        }
    }

    /// Pre-u8 behavior, retained as the graceful-degrade path: copy the OCR
    /// text to the clipboard and toast.
    async fn auto_copy_fallback(&self, ocr_text: &str, request_id: &str) -> IpcResponse {
        use pixelens_ipc::Command;
        let req = IpcRequest {
            command: Command::Copy,
            request_id: format!("{request_id}-fallback"),
            payload: serde_json::to_value(CopyPayload {
                text: ocr_text.to_string(),
            })
            .expect("serialize copy payload"),
        };
        self.handle_copy(&req)
    }

    /// OCR a captured image file on disk (pipeline / slurp+grim path).
    fn ocr_text_path(&self, path: &Path) -> String {
        match &self.state.ocr {
            Some(engine) => match engine.extract_from_path(path) {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        path = %path.display(),
                        "OCR failed on captured image; returning empty text"
                    );
                    String::new()
                }
            },
            None => {
                tracing::debug!("OCR engine not available; skipping text extraction");
                String::new()
            }
        }
    }

    /// OCR an in-memory capture image (portal path — no file written).
    fn ocr_text(&self, image: &CaptureImage) -> String {
        match &self.state.ocr {
            Some(engine) => match engine.extract_text(image) {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "OCR failed on in-memory capture; returning empty text"
                    );
                    String::new()
                }
            },
            None => {
                tracing::debug!("OCR engine not available; skipping text extraction");
                String::new()
            }
        }
    }

    fn handle_status(&self, request: &IpcRequest) -> IpcResponse {
        let display_str = self
            .state
            .display
            .map(|d| format!("{:?}", d).to_lowercase())
            .unwrap_or_else(|| "windows".to_string());
        let payload = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "display": display_str,
            "pipeline_ready": self.state.pipeline.is_some(),
        });
        match IpcResponse::ok(request.request_id.clone(), payload) {
            Ok(r) => r,
            Err(e) => IpcResponse::error(request.request_id.clone(), e.to_string()),
        }
    }

    fn handle_stop(&self, request: &IpcRequest) -> IpcResponse {
        tracing::info!("stop requested via IPC; exiting");
        // Spawn a short task to exit after responding so the response
        // makes it back to the CLI first.
        tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            std::process::exit(0);
        });
        let payload = serde_json::json!({ "stopping": true });
        IpcResponse::ok(request.request_id.clone(), payload)
            .unwrap_or_else(|e| IpcResponse::error(request.request_id.clone(), e.to_string()))
    }

    fn handle_not_implemented(&self, request: &IpcRequest, what: &str) -> IpcResponse {
        IpcResponse::error(
            request.request_id.clone(),
            format!("'{what}' is not yet implemented in this build"),
        )
        .with_status(ResponseStatus::Error)
    }

    /// UM4: request a re-detect of display outputs before the next grab.
    /// The flag is consumed by the next grab; we return ok immediately so
    /// the CLI isn't blocked on detection work.
    fn handle_redetect(&self, request: &IpcRequest) -> IpcResponse {
        self.state.request_redetect();
        tracing::info!("redetect requested; outputs will be re-queried on next grab");
        let payload = serde_json::json!({ "redetect": true });
        IpcResponse::ok(request.request_id.clone(), payload)
            .unwrap_or_else(|e| IpcResponse::error(request.request_id.clone(), e.to_string()))
    }

    /// UM4: set a one-shot preview override for the *next* grab only.
    /// The override is consumed by `handle_grab` and reverts to config
    /// afterwards — so a single `setpreview true` does not leave the
    /// daemon permanently in preview mode.
    fn handle_set_preview(&self, request: &IpcRequest) -> IpcResponse {
        let payload: SetPreviewPayload = match serde_json::from_value(request.payload.clone()) {
            Ok(p) => p,
            Err(e) => {
                return IpcResponse::error(
                    request.request_id.clone(),
                    format!("invalid setpreview payload: {e}"),
                )
                .with_status(ResponseStatus::Error);
            }
        };
        self.state.set_preview_override(payload.preview);
        tracing::info!(
            preview = payload.preview,
            "UM4 one-shot preview override set for next grab"
        );
        let payload = serde_json::json!({ "preview": payload.preview, "one_shot": true });
        IpcResponse::ok(request.request_id.clone(), payload)
            .unwrap_or_else(|e| IpcResponse::error(request.request_id.clone(), e.to_string()))
    }

    /// u5: run a prompt through the configured OpenAI-compatible model
    /// (e.g. Ollama) and return its reply. `chat()` is synchronous
    /// (reqwest::blocking), so it runs inside `spawn_blocking` to keep
    /// the async runtime free. Vision models may consume `image_path`.
    async fn handle_ai(&self, request: &IpcRequest) -> IpcResponse {
        let payload: AiPayload = match serde_json::from_value(request.payload.clone()) {
            Ok(p) => p,
            Err(e) => {
                return IpcResponse::error(
                    request.request_id.clone(),
                    format!("invalid ai payload: {e}"),
                )
                .with_status(ResponseStatus::Error);
            }
        };

        // Clone the full config so the blocking task owns a 'static value.
        let cfg = self.state.config.clone();
        let prompt = payload.prompt.clone();
        let image_path = payload.image_path.clone();

        let result = tokio::task::spawn_blocking(move || {
            let client = OpenAiClient::from_config(&cfg);
            let req = AiRequest {
                prompt,
                image_path: image_path.filter(|p| !p.is_empty()),
            };
            client.chat(&req)
        })
        .await;

        let resp = match result {
            Ok(Ok(ai)) => {
                let payload = AiResponsePayload {
                    text: ai.content,
                    model: ai.model,
                };
                IpcResponse::ok(request.request_id.clone(), payload).ok()
            }
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "ai request failed");
                Some(
                    IpcResponse::error(request.request_id.clone(), e.to_string())
                        .with_status(ResponseStatus::Error),
                )
            }
            Err(e) => Some(IpcResponse::error(
                request.request_id.clone(),
                format!("ai task panicked: {e}"),
            )),
        };
        resp.unwrap_or_else(|| {
            IpcResponse::error(request.request_id.clone(), "ai request failed".to_string())
        })
    }

    /// u6: build a web-search URL from text. Pure (no network), so it
    /// runs inline — no `spawn_blocking` needed.
    fn handle_search(&self, request: &IpcRequest) -> IpcResponse {
        let payload: SearchPayload = match serde_json::from_value(request.payload.clone()) {
            Ok(p) => p,
            Err(e) => {
                return IpcResponse::error(
                    request.request_id.clone(),
                    format!("invalid search payload: {e}"),
                )
                .with_status(ResponseStatus::Error);
            }
        };

        let url = build_search_url(&payload.text);
        match IpcResponse::ok(request.request_id.clone(), SearchResponsePayload { url }) {
            Ok(r) => r.with_status(ResponseStatus::Ok),
            Err(e) => IpcResponse::error(
                request.request_id.clone(),
                format!("serialize search response: {e}"),
            ),
        }
    }

    /// u7: copy text to the system clipboard via the display-appropriate
    /// backend. Pure (no network), so it runs inline. The status string
    /// is surfaced via [`AiResponsePayload`] (text=status, model="clipboard").
    /// Also fires a desktop notification on success (M7).
    fn handle_copy(&self, request: &IpcRequest) -> IpcResponse {
        let payload: CopyPayload = match serde_json::from_value(request.payload.clone()) {
            Ok(p) => p,
            Err(e) => {
                return IpcResponse::error(
                    request.request_id.clone(),
                    format!("invalid copy payload: {e}"),
                )
                .with_status(ResponseStatus::Error);
            }
        };

        #[cfg(unix)]
        let result = copy_text(
            &payload.text,
            self.state.display.expect("display server required on Unix"),
        );
        #[cfg(windows)]
        let result = copy_text(&payload.text);

        match result {
            Ok(()) => {
                // M7: fire success notification (non-blocking, best-effort)
                notify_success(NotificationKind::TextCopied.message());

                let status = format!("copied {} bytes to clipboard", payload.text.len());
                match IpcResponse::ok(
                    request.request_id.clone(),
                    AiResponsePayload {
                        text: status,
                        model: "clipboard".to_string(),
                    },
                ) {
                    Ok(r) => r.with_status(ResponseStatus::Ok),
                    Err(e) => IpcResponse::error(
                        request.request_id.clone(),
                        format!("serialize copy response: {e}"),
                    ),
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "clipboard copy failed");
                IpcResponse::error(request.request_id.clone(), e.to_string())
                    .with_status(ResponseStatus::Error)
            }
        }
    }

    /// u6: translate text via the configured model. Mirrors `handle_ai`
    /// but templates a translate instruction into the prompt.
    async fn handle_translate(&self, request: &IpcRequest) -> IpcResponse {
        let payload: TranslatePayload = match serde_json::from_value(request.payload.clone()) {
            Ok(p) => p,
            Err(e) => {
                return IpcResponse::error(
                    request.request_id.clone(),
                    format!("invalid translate payload: {e}"),
                )
                .with_status(ResponseStatus::Error);
            }
        };

        let cfg = self.state.config.clone();
        let prompt = format!(
            "translate the following text to {}:\n\n{}",
            payload.target_lang, payload.text
        );

        let result = tokio::task::spawn_blocking(move || {
            let client = OpenAiClient::from_config(&cfg);
            let req = AiRequest {
                prompt,
                image_path: None,
            };
            client.chat(&req)
        })
        .await;

        let resp = match result {
            Ok(Ok(ai)) => {
                let payload = AiResponsePayload {
                    text: ai.content,
                    model: ai.model,
                };
                IpcResponse::ok(request.request_id.clone(), payload).ok()
            }
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "translate request failed");
                Some(
                    IpcResponse::error(request.request_id.clone(), e.to_string())
                        .with_status(ResponseStatus::Error),
                )
            }
            Err(e) => Some(IpcResponse::error(
                request.request_id.clone(),
                format!("translate task panicked: {e}"),
            )),
        };
        resp.unwrap_or_else(|| {
            IpcResponse::error(
                request.request_id.clone(),
                "translate request failed".to_string(),
            )
        })
    }

    /// u6: search-by-image. Uploads the capture (per config) then builds a
    /// Google Lens URL. The status string is surfaced via
    /// [`AiResponsePayload`] (text=status, model="reverse-image").
    async fn handle_reverse_image(&self, request: &IpcRequest) -> IpcResponse {
        let payload: ReverseImagePayload = match serde_json::from_value(request.payload.clone()) {
            Ok(p) => p,
            Err(e) => {
                return IpcResponse::error(
                    request.request_id.clone(),
                    format!("invalid reverse-image payload: {e}"),
                )
                .with_status(ResponseStatus::Error);
            }
        };

        let cfg = self.state.config.clone();
        let image_path = payload.image_path.clone();

        let result = tokio::task::spawn_blocking(move || {
            let searcher = ReverseImageSearcher::new(cfg);
            searcher.run(&image_path)
        })
        .await;

        let resp = match result {
            Ok(Ok(status)) => {
                let payload = AiResponsePayload {
                    text: status,
                    model: "reverse-image".to_string(),
                };
                IpcResponse::ok(request.request_id.clone(), payload).ok()
            }
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "reverse-image request failed");
                Some(
                    IpcResponse::error(request.request_id.clone(), e.to_string())
                        .with_status(ResponseStatus::Error),
                )
            }
            Err(e) => Some(IpcResponse::error(
                request.request_id.clone(),
                format!("reverse-image task panicked: {e}"),
            )),
        };
        resp.unwrap_or_else(|| {
            IpcResponse::error(
                request.request_id.clone(),
                "reverse-image request failed".to_string(),
            )
        })
    }
}

trait IpcResponseExt {
    fn with_status(self, status: ResponseStatus) -> Self;
}

impl IpcResponseExt for IpcResponse {
    fn with_status(mut self, status: ResponseStatus) -> Self {
        self.status = status;
        self
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use crate::state::OneShot;
    use pixelens_capture::DisplayServer;
    use pixelens_config::Config;
    use pixelens_ipc::{AiResponsePayload, SearchResponsePayload};
    use pixelens_menu::{MenuBackend, MenuChoice, MenuError};

    /// Injected menu backend that returns a pre-set choice and records that
    /// it was consulted (with the OCR text it received).
    struct StubBackend {
        choice: MenuChoice,
        seen: Arc<Mutex<Vec<String>>>,
    }

    impl MenuBackend for StubBackend {
        fn name(&self) -> &'static str {
            "stub"
        }

        fn show_menu(&self, ocr_text: &str) -> Result<MenuChoice, MenuError> {
            self.seen.lock().unwrap().push(ocr_text.to_string());
            Ok(self.choice)
        }
    }

    fn test_state(backend: StubBackend) -> DaemonState {
        DaemonState {
            display: if cfg!(unix) { Some(DisplayServer::Wayland) } else { None },
            pipeline: None,
            ocr: None,
            config: Config::default(),
            portal_backend: None,
            one_shot: Arc::new(Mutex::new(OneShot::default())),
            menu_override: Some(Arc::new(backend)),
        }
    }

    #[tokio::test]
    async fn decide_action_copy_dispatches_clipboard() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let state = test_state(StubBackend {
            choice: MenuChoice::Copy,
            seen: seen.clone(),
        });
        let dispatcher = Dispatcher::new(Arc::new(state));

        let resp = dispatcher.decide_action("hello world", "req-1").await;

        // Menu was consulted with the OCR text.
        assert_eq!(
            seen.lock().unwrap().as_slice(),
            &["hello world".to_string()]
        );
        // Copy handler ran. On Unix there is no clipboard backend in headless
        // env so it returns Error; on Windows arboard uses native API and
        // returns Ok. Either is valid — the point is the menu→handler wiring.
        #[cfg(unix)]
        assert_eq!(resp.status, ResponseStatus::Error);
        #[cfg(windows)]
        assert_eq!(resp.status, ResponseStatus::Ok);
        let msg = resp
            .payload
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        #[cfg(unix)]
        assert!(msg.to_lowercase().contains("clipboard"));
    }

    #[tokio::test]
    async fn decide_action_search_builds_url_request() {
        let state = test_state(StubBackend {
            choice: MenuChoice::Search,
            seen: Arc::new(Mutex::new(Vec::new())),
        });
        let dispatcher = Dispatcher::new(Arc::new(state));

        let resp = dispatcher.decide_action("find me", "req-2").await;
        // Search handler returns Ok with the resulting payload.
        assert_eq!(resp.status, ResponseStatus::Ok);
        // The dispatched command was Search → URL search payload built.
        let payload: SearchResponsePayload = serde_json::from_value(resp.payload).unwrap();
        assert!(payload.url.contains("find%20me") || payload.url.contains("find me"));
    }

    #[tokio::test]
    async fn decide_action_cancel_returns_no_side_effect() {
        let state = test_state(StubBackend {
            choice: MenuChoice::Cancel,
            seen: Arc::new(Mutex::new(Vec::new())),
        });
        let dispatcher = Dispatcher::new(Arc::new(state));

        let resp = dispatcher.decide_action("anything", "req-3").await;
        assert_eq!(resp.status, ResponseStatus::Ok);
        let payload: AiResponsePayload = serde_json::from_value(resp.payload).unwrap();
        assert!(payload.text.contains("cancel"));
    }
}
