//! Request dispatcher: maps a [`Command`] to its handler.
//!
//! v1 only wires `Grab` to the real capture pipeline. Other commands
//! return a clear "not yet implemented" error so the CLI sees a
//! structured failure rather than a timeout.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use pixelens_capture::GrabOutcome;
use pixelens_ipc::{GrabResponsePayload, IpcRequest, IpcResponse, ResponseStatus};

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
        }
    }

    async fn handle_grab(&self, request: &IpcRequest) -> IpcResponse {
        let Some(pipeline) = &self.state.pipeline else {
            return IpcResponse::error(
                request.request_id.clone(),
                "capture pipeline not initialised: install slurp and grim, then restart the daemon",
            );
        };

        // Run the pipeline on a blocking thread: slurp / grim do real
        // I/O and may block for several seconds while the overlay is
        // on screen. We need a Sync handle to the pipeline to send it
        // across threads; since GrabPipeline stores only `Box<dyn ...>`
        // it's already Sync, so we can build a thin Arc-like wrapper
        // via `pipeline.clone_handle()` — but the simpler path is to
        // hold the pipeline behind an Arc at the state level.
        // We do the latter: see state::DaemonState.
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
                // M5: run OCR on the captured PNG and attach the text to
                // the response. OCR failure is non-fatal — the capture
                // already succeeded, so we return whatever text we got
                // (possibly empty) rather than failing the whole grab.
                let text = match &self.state.ocr {
                    Some(engine) => match engine.extract_from_path(&path) {
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
                };

                let payload = GrabResponsePayload {
                    path: path.to_string_lossy().into_owned(),
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
            GrabOutcome::Cancelled => IpcResponse::cancelled(request.request_id.clone()),
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
