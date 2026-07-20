use serde::{Deserialize, Serialize};

/// A request to the AI backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    /// The prompt / instruction sent to the model.
    pub prompt: String,
    /// Optional path to a captured image (for vision models).
    pub image_path: Option<String>,
}

/// A response from the AI backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    /// Generated content.
    pub content: String,
    /// Model that produced the response.
    pub model: String,
}

impl AiResponse {
    pub fn new(content: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            model: model.into(),
        }
    }
}
