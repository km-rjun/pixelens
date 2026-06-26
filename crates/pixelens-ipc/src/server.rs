use std::path::PathBuf;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use pixelens_actions::get_handler;
use pixelens_capture::{check_tools as check_capture_tools, detect_backend};
use pixelens_config::Config;
use pixelens_ocr::{check_tools as check_ocr_tools, create_engine};

use crate::protocol::{Request, Response};
use crate::IpcError;

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
        Request::Grab { search, ai } => match detect_backend() {
            Ok(backend) => match backend.select_region() {
                Ok(region) => match backend.capture(&region) {
                    Ok(result) => {
                        let image_path = result.image_path.clone();

                        if let Some(prompt) = ai {
                            let config = Config::load();
                            let client = pixelens_actions::ai::OpenAiClient::new(
                                config.api_endpoint,
                                config.api_key,
                                config.model.clone(),
                            );
                            let ai_request = pixelens_common::AiRequest {
                                prompt,
                                image_path: Some(image_path.clone()),
                            };
                            match client.chat(&ai_request) {
                                Ok(response) => Response::GrabResult {
                                    image_path,
                                    text: None,
                                    ai_response: Some(response.content),
                                },
                                Err(e) => Response::Error(e.to_string()),
                            }
                        } else if search {
                            let config = Config::load();
                            let client = pixelens_actions::ai::OpenAiClient::new(
                                config.api_endpoint,
                                config.api_key,
                                config.model.clone(),
                            );
                            let ocr_result = create_engine().and_then(|engine| {
                                engine.perform_ocr(&image_path, &config.ocr_language)
                            });
                            match ocr_result {
                                Ok(result) => {
                                    let ai_request = pixelens_common::AiRequest {
                                        prompt: format!("Search the web for: {}", result.text),
                                        image_path: Some(image_path.clone()),
                                    };
                                    match client.chat(&ai_request) {
                                        Ok(response) => Response::GrabResult {
                                            image_path,
                                            text: Some(result.text),
                                            ai_response: Some(response.content),
                                        },
                                        Err(e) => Response::Error(e.to_string()),
                                    }
                                }
                                Err(e) => Response::Error(e.to_string()),
                            }
                        } else {
                            let config = Config::load();
                            let text = create_engine().and_then(|engine| {
                                engine.perform_ocr(&image_path, &config.ocr_language)
                            });
                            Response::GrabResult {
                                image_path,
                                text: text.ok().map(|r| r.text),
                                ai_response: None,
                            }
                        }
                    }
                    Err(e) => Response::Error(e.to_string()),
                },
                Err(e) => Response::Error(e.to_string()),
            },
            Err(e) => Response::Error(e.to_string()),
        },
        Request::Ocr {
            image_path,
            language,
        } => match create_engine() {
            Ok(engine) => match engine.perform_ocr(&image_path, &language) {
                Ok(result) => Response::OcrResult(result),
                Err(e) => Response::Error(e.to_string()),
            },
            Err(e) => Response::Error(e.to_string()),
        },
        Request::Ai { prompt, image_path } => {
            let config = Config::load();
            let client = pixelens_actions::ai::OpenAiClient::new(
                config.api_endpoint,
                config.api_key,
                config.model.clone(),
            );
            let request = pixelens_common::AiRequest { prompt, image_path };
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
                let payload = pixelens_common::ActionPayload { text, image_path };
                match handler.execute(&payload) {
                    Ok(result) => Response::ActionResult(result),
                    Err(e) => Response::Error(e.to_string()),
                }
            }
            Err(e) => Response::Error(e.to_string()),
        },
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
}
