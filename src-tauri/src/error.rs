use std::fmt;

#[derive(Debug)]
pub enum PixelensError {
    Capture(CaptureError),
    Ocr(OcrError),
    Ai(AiError),
    Config(String),
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum CaptureError {
    ToolNotFound(String),
    RegionCancelled,
    ToolFailed(String),
}

#[derive(Debug)]
pub enum OcrError {
    ToolNotFound(String),
    ToolFailed(String),
    InvalidImage(String),
}

#[derive(Debug)]
pub enum AiError {
    RequestFailed(String),
    InvalidResponse(String),
    Unauthorized,
}

impl fmt::Display for PixelensError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Capture(e) => write!(f, "Capture error: {}", e),
            Self::Ocr(e) => write!(f, "OCR error: {}", e),
            Self::Ai(e) => write!(f, "AI error: {}", e),
            Self::Config(e) => write!(f, "Config error: {}", e),
            Self::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolNotFound(tool) => write!(f, "{} not found", tool),
            Self::RegionCancelled => write!(f, "Region selection cancelled"),
            Self::ToolFailed(msg) => write!(f, "Capture failed: {}", msg),
        }
    }
}

impl fmt::Display for OcrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolNotFound(tool) => write!(f, "{} not found", tool),
            Self::ToolFailed(msg) => write!(f, "OCR failed: {}", msg),
            Self::InvalidImage(msg) => write!(f, "Invalid image: {}", msg),
        }
    }
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestFailed(msg) => write!(f, "Request failed: {}", msg),
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            Self::Unauthorized => write!(f, "Unauthorized - check API key"),
        }
    }
}

impl From<std::io::Error> for PixelensError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<CaptureError> for String {
    fn from(e: CaptureError) -> Self {
        e.to_string()
    }
}

impl From<OcrError> for String {
    fn from(e: OcrError) -> Self {
        e.to_string()
    }
}

impl From<AiError> for String {
    fn from(e: AiError) -> Self {
        e.to_string()
    }
}

impl From<PixelensError> for String {
    fn from(e: PixelensError) -> Self {
        e.to_string()
    }
}
