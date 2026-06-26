use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::actions::get_handler;
use crate::capture::{check_tools as check_capture_tools, detect_backend};
use crate::config::Config;
use crate::ocr::{check_tools as check_ocr_tools, clean_ocr_output, create_engine};

use super::protocol::{Request, Response};
use super::IpcError;

static CAPTURING: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
pub struct IpcServer {
    socket_path: PathBuf,
}

impl IpcServer {
    pub fn new() -> Self {
        let socket_dir = dirs::runtime_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("pixelens");
        Self {
            socket_path: socket_dir.join("pixelens.sock"),
        }
    }

    pub async fn start(&self) -> Result<(), IpcError> {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        log::info!("IPC server listening on {:?}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream).await {
                            log::error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("Accept error: {}", e);
                }
            }
        }
    }

    pub fn stop(&self) {
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }
        log::info!("IPC server stopped");
    }
}

impl Default for IpcServer {
    fn default() -> Self {
        Self::new()
    }
}

async fn handle_connection(stream: UnixStream) -> Result<(), IpcError> {
    let (reader, mut writer) = stream.into_split();
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|e| IpcError::ReadFailed(e.to_string()))?
    {
        let request: Request = serde_json::from_str(&line)?;
        let response = handle_request(request).await;
        let response_json = serde_json::to_string(&response)?;
        writer
            .write_all(response_json.as_bytes())
            .await
            .map_err(|e| IpcError::WriteFailed(e.to_string()))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| IpcError::WriteFailed(e.to_string()))?;
    }

    Ok(())
}

async fn handle_request(request: Request) -> Response {
    match request {
        Request::Ping => Response::Pong,
        Request::Status => {
            let capture_missing = check_capture_tools();
            let ocr_missing = check_ocr_tools();
            Response::Status {
                running: true,
                capture_missing,
                ocr_missing,
            }
        }
        Request::Stop => {
            log::info!("Stop request received");
            Response::Stopped
        }
        Request::CheckTools => {
            let capture_missing = check_capture_tools();
            let ocr_missing = check_ocr_tools();
            Response::ToolsStatus {
                capture_missing,
                ocr_missing,
            }
        }
        Request::GetConfig => {
            let config = Config::load();
            Response::Config {
                api_endpoint: config.api_endpoint,
                model: config.model,
                ocr_language: config.ocr_language,
            }
        }
        Request::Grab { search, ai } => {
            if CAPTURING.swap(true, Ordering::SeqCst) {
                return Response::Error("Capture already in progress".to_string());
            }

            let result = handle_grab(search, ai).await;

            CAPTURING.store(false, Ordering::SeqCst);
            result
        }
        Request::Ocr {
            image_path,
            language,
        } => match create_engine() {
            Ok(engine) => match engine.perform_ocr(&image_path, &language) {
                Ok(result) => {
                    let mut r = result;
                    r.text = clean_ocr_output(&r.text);
                    Response::OcrResult(r)
                }
                Err(e) => Response::Error(e.to_string()),
            },
            Err(e) => Response::Error(e.to_string()),
        },
        Request::Ai { prompt, image_path } => {
            let config = Config::load();
            let client = crate::actions::ai::OpenAiClient::new(
                config.api_endpoint,
                config.api_key,
                config.model.clone(),
            );
            let request = crate::types::AiRequest { prompt, image_path };
            match client.chat(&request) {
                Ok(response) => Response::AiResult {
                    content: response.content,
                    model: response.model,
                },
                Err(e) => Response::Error(e.to_string()),
            }
        }
        Request::Action {
            action,
            text,
            image_path,
        } => match get_handler(&action) {
            Ok(handler) => {
                let payload = crate::types::ActionPayload { text, image_path };
                match handler.execute(&payload) {
                    Ok(result) => Response::ActionResult(result),
                    Err(e) => Response::Error(e.to_string()),
                }
            }
            Err(e) => Response::Error(e.to_string()),
        },
    }
}

async fn handle_grab(_search: bool, ai: Option<String>) -> Response {
    let backend = match detect_backend() {
        Ok(b) => b,
        Err(e) => return Response::Error(e.to_string()),
    };

    let region = match backend.select_region() {
        Ok(r) => r,
        Err(crate::error::CaptureError::RegionCancelled) => {
            return Response::Error("Region selection cancelled".to_string());
        }
        Err(e) => return Response::Error(e.to_string()),
    };

    let result = match backend.capture(&region) {
        Ok(r) => r,
        Err(e) => return Response::Error(e.to_string()),
    };

    let image_path = result.image_path.clone();

    let ocr_result = {
        let config = Config::load();
        create_engine().and_then(|engine| engine.perform_ocr(&image_path, &config.ocr_language))
    };

    let extracted_text = match ocr_result {
        Ok(r) => Some(clean_ocr_output(&r.text)),
        Err(e) => {
            log::warn!("OCR failed: {}", e);
            None
        }
    };

    if let Some(prompt) = ai {
        let config = Config::load();
        let client = crate::actions::ai::OpenAiClient::new(
            config.api_endpoint,
            config.api_key,
            config.model.clone(),
        );
        let ai_request = crate::types::AiRequest {
            prompt,
            image_path: Some(image_path.clone()),
        };
        match client.chat(&ai_request) {
            Ok(response) => Response::GrabResult {
                image_path,
                text: extracted_text,
                ai_response: Some(response.content),
            },
            Err(e) => Response::Error(e.to_string()),
        }
    } else {
        Response::GrabResult {
            image_path,
            text: extracted_text,
            ai_response: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = IpcServer::new();
        assert!(server.socket_path.to_string_lossy().contains("pixelens"));
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let response = handle_request(Request::Ping).await;
        assert!(matches!(response, Response::Pong));
    }

    #[tokio::test]
    async fn test_handle_status() {
        let response = handle_request(Request::Status).await;
        assert!(matches!(response, Response::Status { .. }));
    }

    #[tokio::test]
    async fn test_handle_stop() {
        let response = handle_request(Request::Stop).await;
        assert!(matches!(response, Response::Stopped));
    }

    #[test]
    fn test_concurrent_capture_guard() {
        let was_capturing = CAPTURING.swap(true, Ordering::SeqCst);
        assert!(!was_capturing);
        let was_capturing = CAPTURING.swap(true, Ordering::SeqCst);
        assert!(was_capturing);
        CAPTURING.store(false, Ordering::SeqCst);
    }
}
