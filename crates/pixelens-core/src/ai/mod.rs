pub mod provider_error;

use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::error::{AiError, RateLimitKind};
use crate::types::{AiRequest, AiResponse};

use provider_error::{parse_429_response, parse_retry_after};

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

const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 1000;

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

    pub fn from_config(config: &Config) -> Self {
        Self {
            endpoint: config.api_endpoint.clone(),
            api_key: config.api_key.clone(),
            model: config.model.clone(),
        }
    }

    fn validate_api_key(&self) -> Result<(), AiError> {
        if self.api_key.is_empty() {
            let config_path = Config::config_path().to_string_lossy().to_string();
            return Err(AiError::Unauthorized {
                endpoint: self.endpoint.clone(),
                config_path,
            });
        }
        Ok(())
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

    fn do_request(&self, chat_request: &ChatRequest) -> Result<String, AiError> {
        let url = format!("{}/chat/completions", self.endpoint);
        let response = ureq::post(&url)
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send_json(chat_request)
            .map_err(|e| {
                let msg = format!("{}", e);
                if msg.contains("401") || msg.contains("Unauthorized") {
                    let config_path = Config::config_path().to_string_lossy().to_string();
                    AiError::Unauthorized {
                        endpoint: self.endpoint.clone(),
                        config_path,
                    }
                } else if msg.contains("429") {
                    AiError::RateLimited {
                        kind: RateLimitKind::Temporary {
                            retry_after_secs: None,
                        },
                    }
                } else {
                    AiError::RequestFailed(msg)
                }
            })?;

        let status = response.status();
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .and_then(parse_retry_after);

        let body = response
            .into_body()
            .read_to_string()
            .map_err(|e| AiError::RequestFailed(format!("Read response: {}", e)))?;

        if status == 429 {
            let kind = parse_429_response(&body);
            let kind = match kind {
                RateLimitKind::Temporary { .. } => RateLimitKind::Temporary {
                    retry_after_secs: retry_after,
                },
                other => other,
            };
            return Err(AiError::RateLimited { kind });
        }

        Ok(body)
    }

    pub fn chat(&self, request: &AiRequest) -> Result<AiResponse, AiError> {
        self.validate_api_key()?;

        let chat_request = self.build_request(request);
        let mut attempts = 0;

        loop {
            attempts += 1;
            match self.do_request(&chat_request) {
                Ok(body) => return self.parse_response(&body),
                Err(AiError::RateLimited { kind }) => {
                    if kind == RateLimitKind::QuotaExhausted || attempts >= MAX_RETRIES {
                        return Err(AiError::RateLimited { kind });
                    }
                    let delay = match &kind {
                        RateLimitKind::Temporary {
                            retry_after_secs: Some(secs),
                        } => Duration::from_secs(*secs),
                        _ => {
                            let base = BASE_DELAY_MS * 2u64.pow(attempts - 1);
                            let jitter = rand_delay(base);
                            Duration::from_millis(jitter)
                        }
                    };
                    thread::sleep(delay);
                }
                Err(e) => return Err(e),
            }
        }
    }
}

fn rand_delay(base_ms: u64) -> u64 {
    let jitter = (base_ms as f64 * 0.2) as u64;
    let offset = (base_ms / 5).min(jitter);
    base_ms - offset + (fastrand::u64(0..offset * 2 + 1))
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
    fn test_empty_api_key_rejected() {
        let client = OpenAiClient::new(
            "https://api.openai.com/v1".to_string(),
            String::new(),
            "gpt-4o".to_string(),
        );
        let request = AiRequest {
            prompt: "test".to_string(),
            image_path: None,
        };
        let result = client.chat(&request);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("API key is missing"));
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
            "gpt-40".to_string(),
        );

        let result = client.parse_response("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_rand_delay_bounds() {
        for _ in 0..100 {
            let delay = rand_delay(1000);
            assert!(
                (800..=1200).contains(&delay),
                "delay out of range: {}",
                delay
            );
        }
    }
}
