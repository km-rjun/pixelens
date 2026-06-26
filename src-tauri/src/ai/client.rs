use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;

use super::AiProvider;
use crate::error::AiError;
use crate::types::{AiRequest, AiResponse};

#[derive(Serialize)]
struct Message {
    role: String,
    content: serde_json::Value,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

pub struct OpenAiClient {
    endpoint: String,
    api_key: String,
    model: String,
}

impl OpenAiClient {
    pub fn new(endpoint: String, api_key: String, model: String) -> Self {
        Self {
            endpoint,
            api_key,
            model,
        }
    }
}

impl AiProvider for OpenAiClient {
    fn chat(&self, request: &AiRequest) -> Result<AiResponse, AiError> {
        let mut content = serde_json::Value::Array(vec![]);

        if let Some(ref path) = request.image_path {
            let image_data =
                fs::read(path).map_err(|e| AiError::RequestFailed(format!("Read image: {}", e)))?;
            let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);

            let image_content = serde_json::json!({
                "type": "image_url",
                "image_url": {
                    "url": format!("data:image/png;base64,{}", base64_image)
                }
            });

            content.as_array_mut().unwrap().push(image_content);
        }

        let text_content = serde_json::json!({
            "type": "text",
            "text": request.prompt
        });
        content.as_array_mut().unwrap().push(text_content);

        let chat_request = ChatRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content,
            }],
        };

        let url = format!("{}/chat/completions", self.endpoint);
        let response = ureq::post(&url)
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send_json(&chat_request)
            .map_err(|e| AiError::RequestFailed(format!("{}", e)))?;

        let body = response
            .into_body()
            .read_to_string()
            .map_err(|e| AiError::RequestFailed(format!("Read response: {}", e)))?;

        let chat_response: ChatResponse =
            serde_json::from_str(&body).map_err(|e| AiError::InvalidResponse(format!("{}", e)))?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AiError::InvalidResponse("No choices in response".to_string()))?;

        Ok(AiResponse {
            content,
            model: self.model.clone(),
        })
    }

    fn name(&self) -> &str {
        "openai"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OpenAiClient::new(
            "https://api.openai.com/v1".to_string(),
            "test-key".to_string(),
            "gpt-4o".to_string(),
        );
        assert_eq!(client.name(), "openai");
    }
}
