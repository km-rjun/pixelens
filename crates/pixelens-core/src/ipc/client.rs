use std::path::PathBuf;

use crate::types::ActionType;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use super::protocol::{Request, Response};
use super::IpcError;

pub struct IpcClient {
    socket_path: PathBuf,
}

impl IpcClient {
    pub fn new() -> Self {
        let socket_dir = dirs::runtime_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("pixelens");
        Self {
            socket_path: socket_dir.join("pixelens.sock"),
        }
    }

    pub async fn send(&self, request: Request) -> Result<Response, IpcError> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;

        let request_json = serde_json::to_string(&request)?;
        stream
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| IpcError::WriteFailed(e.to_string()))?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| IpcError::WriteFailed(e.to_string()))?;

        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        if let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| IpcError::ReadFailed(e.to_string()))?
        {
            let response: Response = serde_json::from_str(&line)?;
            Ok(response)
        } else {
            Err(IpcError::ReadFailed("Connection closed".to_string()))
        }
    }

    pub async fn ping(&self) -> Result<bool, IpcError> {
        match self.send(Request::Ping).await {
            Ok(Response::Pong) => Ok(true),
            Ok(_) => Ok(false),
            Err(IpcError::ConnectionFailed(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub async fn status(&self) -> Result<(bool, Vec<String>, Vec<String>), IpcError> {
        match self.send(Request::Status).await? {
            Response::Status {
                running,
                capture_missing,
                ocr_missing,
            } => Ok((running, capture_missing, ocr_missing)),
            Response::Error(e) => Err(IpcError::ServerError(e)),
            _ => Err(IpcError::InvalidResponse),
        }
    }

    pub async fn stop(&self) -> Result<(), IpcError> {
        match self.send(Request::Stop).await? {
            Response::Stopped => Ok(()),
            Response::Error(e) => Err(IpcError::ServerError(e)),
            _ => Err(IpcError::InvalidResponse),
        }
    }

    pub async fn grab(
        &self,
        search: bool,
        ai: Option<&str>,
    ) -> Result<(String, Option<String>, Option<String>), IpcError> {
        match self
            .send(Request::Grab {
                search,
                ai: ai.map(|s| s.to_string()),
            })
            .await?
        {
            Response::GrabResult {
                image_path,
                text,
                ai_response,
            } => Ok((image_path, text, ai_response)),
            Response::Error(e) => Err(IpcError::ServerError(e)),
            _ => Err(IpcError::InvalidResponse),
        }
    }

    pub async fn ocr(
        &self,
        image_path: &str,
        language: &str,
    ) -> Result<crate::types::OcrResult, IpcError> {
        match self
            .send(Request::Ocr {
                image_path: image_path.to_string(),
                language: language.to_string(),
            })
            .await?
        {
            Response::OcrResult(result) => Ok(result),
            Response::Error(e) => Err(IpcError::ServerError(e)),
            _ => Err(IpcError::InvalidResponse),
        }
    }

    pub async fn ai(
        &self,
        prompt: &str,
        image_path: Option<&str>,
    ) -> Result<(String, String), IpcError> {
        match self
            .send(Request::Ai {
                prompt: prompt.to_string(),
                image_path: image_path.map(|s| s.to_string()),
            })
            .await?
        {
            Response::AiResult { content, model } => Ok((content, model)),
            Response::Error(e) => Err(IpcError::ServerError(e)),
            _ => Err(IpcError::InvalidResponse),
        }
    }

    pub async fn action(
        &self,
        action: ActionType,
        text: &str,
        image_path: Option<&str>,
    ) -> Result<String, IpcError> {
        match self
            .send(Request::Action {
                action,
                text: text.to_string(),
                image_path: image_path.map(|s| s.to_string()),
            })
            .await?
        {
            Response::ActionResult(result) => Ok(result),
            Response::Error(e) => Err(IpcError::ServerError(e)),
            _ => Err(IpcError::InvalidResponse),
        }
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = IpcClient::new();
        assert!(client.socket_path.to_string_lossy().contains("pixelens"));
    }

    #[test]
    fn test_socket_path() {
        let client = IpcClient::new();
        assert!(client.socket_path.ends_with("pixelens.sock"));
    }
}
