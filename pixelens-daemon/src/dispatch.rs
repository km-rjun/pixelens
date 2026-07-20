//! Request dispatcher: maps a [`Command`] to its handler.
//!
//! v1 only wires `Grab` to the real capture pipeline. Other commands
//! return a clear "not yet implemented" error so the CLI sees a
//! structured failure rather than a timeout.

use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use pixelens_capture::GrabOutcome;
use pixelens_core::{
    CaptureError, CaptureImage, CaptureProvider, CaptureRequest, OcrEngine, PixelensError, Rect,
};
use pixelens_ipc::{
    GrabResponsePayload, IpcRequest, IpcResponse, ResponseStatus, SetPreviewPayload,
};
use pixelens_notify::{NotificationKind, Notifier, NotifySend};

use crate::clipboard::copy_text;
use crate::state::DaemonState;

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
                return IpcResponse::error(
                        request.request_id.clone(),
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
                    return IpcResponse::error(request.request_id.clone(), e.to_string());
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

        // M7: complete the core loop. The capture + OCR already
        // succeeded above; now push the result to the user:
        //  - non-empty text → copy to clipboard + "text copied" toast
        //  - empty text      → "no text found" toast (NOT an error)
        // Both clipboard and notify are best-effort and must never
        // turn a successful capture into a failed grab.
        let notifier = NotifySend::new();
        if text.is_empty() {
            tracing::info!("grab produced no text; notifying 'no text found'");
            if let Err(e) = notifier.send(NotificationKind::NoTextFound) {
                tracing::warn!(error = %e, "failed to send 'no text found' notification");
            }
        } else {
            match copy_text(&text, self.state.display) {
                Ok(()) => tracing::info!("copied extracted text to clipboard"),
                Err(e) => tracing::warn!(
                    error = %e,
                    "clipboard copy failed; continuing without clipboard text"
                ),
            }
            if let Err(e) = notifier.send(NotificationKind::TextCopied) {
                tracing::warn!(error = %e, "failed to send 'text copied' notification");
            }
        }

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
        let payload = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "display": format!("{:?}", self.state.display).to_lowercase(),
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
