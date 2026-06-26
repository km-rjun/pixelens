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

#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitKind {
    Temporary { retry_after_secs: Option<u64> },
    QuotaExhausted,
}

impl std::fmt::Display for RateLimitKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Temporary { retry_after_secs } => match retry_after_secs {
                Some(secs) => write!(f, "Rate limited. Retry after {} seconds", secs),
                None => write!(f, "Rate limited. Retry later"),
            },
            Self::QuotaExhausted => write!(
                f,
                "Quota or usage limit reached. Check billing at your provider"
            ),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Authentication failed for {endpoint}: API key is missing, invalid, or rejected. Set PIXELENS_API_KEY environment variable or configure api_key in {config_path}")]
    Unauthorized {
        endpoint: String,
        config_path: String,
    },

    #[error("Rate limit: {kind}")]
    RateLimited { kind: RateLimitKind },

    #[error("Retry exhausted after {attempts} attempts")]
    RetryExhausted { attempts: u32 },
}
