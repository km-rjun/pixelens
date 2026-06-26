#[derive(Debug, thiserror::Error)]
pub enum PixelensError {
    #[error("Capture error: {0}")]
    Capture(#[from] CaptureError),

    #[error("OCR error: {0}")]
    Ocr(#[from] OcrError),

    #[error("AI error: {0}")]
    Ai(#[from] AiError),

    #[error("Config error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("{0} not found")]
    ToolNotFound(String),

    #[error("Region selection cancelled")]
    RegionCancelled,

    #[error("Capture failed: {0}")]
    ToolFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum OcrError {
    #[error("{0} not found")]
    ToolNotFound(String),

    #[error("OCR failed: {0}")]
    ToolFailed(String),

    #[error("Invalid image: {0}")]
    InvalidImage(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Unauthorized - check API key")]
    Unauthorized,
}
