use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;

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

    fn build_request(&self, request: &AiRequest) -> ChatRequest {
        let mut content = serde_json::Value::Array(vec![]);

        if let Some(ref path) = request.image_path {
            if let Ok(image_data) = fs::read(path) {
                let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);
                let image_content = serde_json::json!({
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:image/png;base64,{}", base64_image)
                    }
                });
                content.as_array_mut().unwrap().push(image_content);
            }
        }

        let text_content = serde_json::json!({
            "type": "text",
            "text": request.prompt
        });
        content.as_array_mut().unwrap().push(text_content);

        ChatRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content,
            }],
        }
    }

    fn parse_response(&self, body: &str) -> Result<AiResponse, AiError> {
        let chat_response: ChatResponse =
            serde_json::from_str(body).map_err(|e| AiError::InvalidResponse(format!("{}", e)))?;

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

    pub fn chat(&self, request: &AiRequest) -> Result<AiResponse, AiError> {
        let chat_request = self.build_request(request);

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

        self.parse_response(&body)
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
        assert_eq!(client.model, "gpt-4o");
    }

    #[test]
    fn test_build_request_text_only() {
        let client = OpenAiClient::new(
            "https://api.openai.com/v1".to_string(),
            "test-key".to_string(),
            "gpt-4o".to_string(),
        );

        let request = AiRequest {
            prompt: "Hello world".to_string(),
            image_path: None,
        };

        let chat_request = client.build_request(&request);
        assert_eq!(chat_request.model, "gpt-4o");
        assert_eq!(chat_request.messages.len(), 1);

        let content = &chat_request.messages[0].content;
        let arr = content.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["type"], "text");
        assert_eq!(arr[0]["text"], "Hello world");
    }

    #[test]
    fn test_parse_response_valid() {
        let client = OpenAiClient::new(
            "https://api.openai.com/v1".to_string(),
            "test-key".to_string(),
            "gpt-4o".to_string(),
        );

        let response_json = r#"{
            "choices": [{
                "message": {
                    "content": "Hello from AI"
                }
            }]
        }"#;

        let result = client.parse_response(response_json).unwrap();
        assert_eq!(result.content, "Hello from AI");
        assert_eq!(result.model, "gpt-4o");
    }

    #[test]
    fn test_parse_response_empty_choices() {
        let client = OpenAiClient::new(
            "https://api.openai.com/v1".to_string(),
            "test-key".to_string(),
            "gpt-4o".to_string(),
        );

        let response_json = r#"{"choices": []}"#;
        let result = client.parse_response(response_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_response_invalid_json() {
        let client = OpenAiClient::new(
            "https://api.openai.com/v1".to_string(),
            "test-key".to_string(),
            "gpt-4o".to_string(),
        );

        let result = client.parse_response("not json");
        assert!(result.is_err());
    }
}
