pub mod client;
pub mod protocol;
pub mod server;

#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Write failed: {0}")]
    WriteFailed(String),

    #[error("Read failed: {0}")]
    ReadFailed(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Invalid response")]
    InvalidResponse,

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = IpcError::ConnectionFailed("test".to_string());
        assert_eq!(error.to_string(), "Connection failed: test");
    }
}
