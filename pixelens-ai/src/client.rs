use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::thread;
use std::time::Duration;

use crate::error::{AiError, RateLimitKind};
use crate::provider_error::{parse_429_response, parse_retry_after};
use crate::types::{AiRequest, AiResponse};

#[derive(Serialize, Debug)]
struct Message {
    role: String,
    content: serde_json::Value,
}

#[derive(Serialize, Debug)]
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

/// Native Ollama `/api/chat` request shape (only the fields we use).
#[derive(Serialize)]
struct OllamaNativeRequest {
    model: String,
    messages: Vec<OllamaNativeMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaNativeMessage {
    role: String,
    content: String,
    images: Vec<String>,
}

#[derive(Deserialize)]
struct OllamaNativeResponse {
    message: ResponseMessage,
}

const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 1000;

const VISION_MODELS: &[&str] = &[
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-4-turbo",
    "gpt-4-vision-preview",
    "claude-3-opus",
    "claude-3-sonnet",
    "claude-3-haiku",
    "claude-3-5-sonnet",
    "llava",
    "bakllava",
];

/// Whether a model name refers to a vision-capable model.
///
/// `endpoint` is consulted so local Ollama hosts (where any pulled model may be
/// vision-capable) relax the named allowlist, while remote providers keep the
/// explicit list (so a text-only model like `gpt-3.5-turbo` is never sent an image).
/// Whether `endpoint` points at a local Ollama server.
///
/// Ollama's OpenAI-compatible `/v1/chat/completions` image path is broken on
/// many builds (it returns 400 "Failed to load image"), while the native
/// `/api/chat` `images` field works. When this returns true we route vision
/// requests through the native API instead.
pub fn is_local_ollama(endpoint: &str) -> bool {
    endpoint.contains(":11434")
        || endpoint.contains("localhost")
        || endpoint.contains("127.0.0.1")
        || endpoint.contains("0.0.0.0")
}

/// Whether a model name refers to a vision-capable model.
///
/// `endpoint` is consulted so local Ollama hosts (where any pulled model may be
/// vision-capable) relax the named allowlist, while remote providers keep the
/// explicit list (so a text-only model like `gpt-3.5-turbo` is never sent an image).
pub fn model_supports_vision(model: &str, endpoint: &str) -> bool {
    let lower = model.to_lowercase();
    let named = VISION_MODELS.iter().any(|m| lower.contains(m));
    // Local Ollama hosts relax the allowlist: any pulled model may be vision-capable,
    // while remote providers keep the explicit list (so text-only models like
    // `gpt-3.5-turbo` are never sent an image).
    named || is_local_ollama(endpoint)
}

/// OpenAI-compatible chat client. `chat()` is **synchronous** (uses
/// `reqwest::blocking`) so the daemon can call it via `spawn_blocking`.
pub struct OpenAiClient {
    endpoint: String,
    api_key: String,
    model: String,
    /// When false, an empty API key is tolerated (e.g. local Ollama/llava).
    require_key: bool,
}

impl OpenAiClient {
    pub fn new(
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
        require_key: bool,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            model: model.into(),
            require_key,
        }
    }

    /// Build from the unified Pixelens config.
    pub fn from_config(config: &pixelens_config::Config) -> Self {
        Self {
            endpoint: config.ai.endpoint.clone(),
            api_key: config.ai.api_key.clone(),
            model: config.ai.model.clone(),
            require_key: config.ai.require_key,
        }
    }

    fn validate_api_key(&self) -> Result<(), AiError> {
        if self.require_key && self.api_key.is_empty() {
            return Err(AiError::Unauthorized {
                endpoint: self.endpoint.clone(),
                config_path: pixelens_config::config_path().to_string_lossy().to_string(),
            });
        }
        Ok(())
    }

    /// Read an image file and return it as standard base64 (no `data:` prefix).
    fn encode_image_base64(path: &str) -> Result<String, AiError> {
        let image_data = fs::read(path)
            .map_err(|e| AiError::RequestFailed(format!("read image {}: {}", path, e)))?;
        Ok(base64::engine::general_purpose::STANDARD.encode(&image_data))
    }

    fn build_request(&self, request: &AiRequest) -> Result<ChatRequest, AiError> {
        let mut content = serde_json::Value::Array(vec![]);

        if let Some(ref path) = request.image_path {
            if !model_supports_vision(&self.model, &self.endpoint) {
                return Err(AiError::RequestFailed(format!(
                    "Model '{}' does not support image input. Use a vision-capable model like gpt-4o, gpt-4-turbo, or claude-3-sonnet",
                    self.model
                )));
            }
            if fs::read(path).is_ok() {
                let base64_image = Self::encode_image_base64(path)?;
                let image_content = serde_json::json!({
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:image/png;base64,{}", base64_image)
                    }
                });
                content.as_array_mut().unwrap().push(image_content);
            } else {
                eprintln!("pixelens-ai: could not read image file: {}", path);
            }
        }

        let text_content = serde_json::json!({
            "type": "text",
            "text": request.prompt
        });
        content.as_array_mut().unwrap().push(text_content);

        Ok(ChatRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content,
            }],
        })
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

        let client = reqwest::blocking::Client::new();
        let builder = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(chat_request);

        let response = builder.send().map_err(|e| {
            let msg = format!("{}", e);
            if msg.contains("401") || msg.contains("Unauthorized") {
                AiError::Unauthorized {
                    endpoint: self.endpoint.clone(),
                    config_path: pixelens_config::config_path().to_string_lossy().to_string(),
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
            .text()
            .map_err(|e| AiError::RequestFailed(format!("Read response: {}", e)))?;

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let kind = parse_429_response(&body);
            let kind = match kind {
                RateLimitKind::Temporary { .. } => RateLimitKind::Temporary {
                    retry_after_secs: retry_after,
                },
                other => other,
            };
            return Err(AiError::RateLimited { kind });
        }

        if !status.is_success() {
            return Err(AiError::RequestFailed(format!(
                "provider returned HTTP {}: {}",
                status, body
            )));
        }

        Ok(body)
    }

    /// POST to the native Ollama `/api/chat` endpoint with base64 images.
    ///
    /// Ollama's OpenAI-compatible `/v1/chat/completions` image path is broken on
    /// many builds (returns 400 "Failed to load image"), but the native API's
    /// `images` field works. We only call this for local Ollama hosts.
    fn do_ollama_native(&self, request: &AiRequest, base64_image: &str) -> Result<String, AiError> {
        // Endpoint may be `http://host:11434/v1` or `http://host:11434`.
        let base = self
            .endpoint
            .trim_end_matches('/')
            .trim_end_matches("/v1")
            .trim_end_matches('/');
        let url = format!("{}/api/chat", base);

        let native = OllamaNativeRequest {
            model: self.model.clone(),
            stream: false,
            messages: vec![OllamaNativeMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
                images: vec![base64_image.to_string()],
            }],
        };

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&native)
            .send()
            .map_err(|e| AiError::RequestFailed(format!("{}", e)))?;

        let status = response.status();
        let body = response
            .text()
            .map_err(|e| AiError::RequestFailed(format!("Read response: {}", e)))?;

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AiError::RateLimited {
                kind: RateLimitKind::Temporary {
                    retry_after_secs: None,
                },
            });
        }
        if !status.is_success() {
            return Err(AiError::RequestFailed(format!(
                "ollama native api returned HTTP {}: {}",
                status, body
            )));
        }

        let parsed: OllamaNativeResponse =
            serde_json::from_str(&body).map_err(|e| AiError::InvalidResponse(format!("{}", e)))?;
        Ok(parsed.message.content)
    }

    /// Send a chat completion request with bounded retries on 429s.
    pub fn chat(&self, request: &AiRequest) -> Result<AiResponse, AiError> {
        self.validate_api_key()?;

        // Local Ollama's OpenAI-compatible image path is broken; route vision
        // requests through the native `/api/chat` `images` field instead.
        if is_local_ollama(&self.endpoint) {
            if let Some(ref path) = request.image_path {
                if !model_supports_vision(&self.model, &self.endpoint) {
                    return Err(AiError::RequestFailed(format!(
                        "Model '{}' does not support image input. Use a vision-capable model like qwen2.5-vl or llava",
                        self.model
                    )));
                }
                let base64_image = Self::encode_image_base64(path)?;
                let content = self.do_ollama_native(request, &base64_image)?;
                return Ok(AiResponse {
                    content,
                    model: self.model.clone(),
                });
            }
        }

        let chat_request = self.build_request(request)?;
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
        let client = OpenAiClient::new("https://api.openai.com/v1", "test-key", "gpt-4o", true);
        assert_eq!(client.model, "gpt-4o");
    }

    #[test]
    fn test_empty_api_key_rejected_when_required() {
        let client = OpenAiClient::new("https://api.openai.com/v1", "", "gpt-4o", true);
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
    fn test_empty_api_key_ok_when_not_required() {
        // Ollama/llava: no key, but do_request still fails (no network in test).
        let client = OpenAiClient::new("http://127.0.0.1:1/v1", "", "llava", false);
        let request = AiRequest {
            prompt: "test".to_string(),
            image_path: None,
        };
        // validate_api_key passes; the network call fails fast — that's expected here.
        let result = client.chat(&request);
        assert!(result.is_err());
        assert!(!result
            .unwrap_err()
            .to_string()
            .contains("API key is missing"));
    }

    #[test]
    fn test_build_request_text_only() {
        let client = OpenAiClient::new("https://api.openai.com/v1", "test-key", "gpt-4o", true);
        let request = AiRequest {
            prompt: "Hello world".to_string(),
            image_path: None,
        };
        let chat_request = client.build_request(&request).unwrap();
        assert_eq!(chat_request.model, "gpt-4o");
        assert_eq!(chat_request.messages.len(), 1);
        let arr = chat_request.messages[0].content.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["type"], "text");
        assert_eq!(arr[0]["text"], "Hello world");
    }

    #[test]
    fn test_build_request_with_image() {
        let tmp = std::env::temp_dir().join("pixelens_test_img.png");
        std::fs::write(&tmp, b"fake png data").unwrap();
        let client = OpenAiClient::new("https://api.openai.com/v1", "test-key", "gpt-4o", true);
        let request = AiRequest {
            prompt: "What is in this image?".to_string(),
            image_path: Some(tmp.to_string_lossy().to_string()),
        };
        let chat_request = client.build_request(&request).unwrap();
        let arr = chat_request.messages[0].content.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["type"], "image_url");
        assert!(arr[0]["image_url"]["url"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,"));
        assert_eq!(arr[1]["type"], "text");
        assert_eq!(arr[1]["text"], "What is in this image?");
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_build_request_image_missing_file() {
        let client = OpenAiClient::new("https://api.openai.com/v1", "test-key", "gpt-4o", true);
        let request = AiRequest {
            prompt: "Describe this".to_string(),
            image_path: Some("/tmp/nonexistent_file_12345.png".to_string()),
        };
        let chat_request = client.build_request(&request).unwrap();
        let arr = chat_request.messages[0].content.as_array().unwrap();
        assert_eq!(arr.len(), 1, "Missing image should fall back to text only");
        assert_eq!(arr[0]["type"], "text");
    }

    #[test]
    fn test_image_rejected_for_non_vision_model() {
        let tmp = std::env::temp_dir().join("pixelens_test_img2.png");
        std::fs::write(&tmp, b"fake png data").unwrap();
        let client = OpenAiClient::new(
            "https://api.openai.com/v1",
            "test-key",
            "gpt-3.5-turbo",
            true,
        );
        let request = AiRequest {
            prompt: "Describe this".to_string(),
            image_path: Some(tmp.to_string_lossy().to_string()),
        };
        let result = client.build_request(&request);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not support image input"));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_model_supports_vision() {
        let remote = "https://api.openai.com/v1";
        // Named allowlist — works regardless of endpoint.
        assert!(model_supports_vision("gpt-4o", remote));
        assert!(model_supports_vision("gpt-4o-mini", remote));
        assert!(model_supports_vision("gpt-4-turbo", remote));
        assert!(model_supports_vision("claude-3-sonnet", remote));
        assert!(model_supports_vision("llava-13b", remote));
        // Remote text-only models stay rejected.
        assert!(!model_supports_vision("gpt-3.5-turbo", remote));
        assert!(!model_supports_vision("text-davinci-003", remote));
        // Local Ollama host relaxes the allowlist for any model name.
        let local = "http://10.0.0.88:11434/v1";
        assert!(model_supports_vision("hermes-qwen3:latest", local));
        assert!(model_supports_vision("qwen3.5:9b", local));
        assert!(model_supports_vision("gpt-3.5-turbo", local));
        let local_loopback = "http://127.0.0.1:11434/v1";
        assert!(model_supports_vision("some-unknown-model", local_loopback));
    }

    #[test]
    fn test_parse_response_valid() {
        let client = OpenAiClient::new("https://api.openai.com/v1", "test-key", "gpt-4o", true);
        let body = r#"{"choices": [{"message": {"content": "Hello from AI"}}]}"#;
        let result = client.parse_response(body).unwrap();
        assert_eq!(result.content, "Hello from AI");
        assert_eq!(result.model, "gpt-4o");
    }

    #[test]
    fn test_parse_response_empty_choices() {
        let client = OpenAiClient::new("https://api.openai.com/v1", "test-key", "gpt-4o", true);
        let result = client.parse_response(r#"{"choices": []}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_response_invalid_json() {
        let client = OpenAiClient::new("https://api.openai.com/v1", "test-key", "gpt-4o", true);
        assert!(client.parse_response("not json").is_err());
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

    #[test]
    fn test_ask_ai_receives_ocr_text_in_prompt() {
        let client = OpenAiClient::new("https://api.openai.com/v1", "test-key", "gpt-4o", true);
        let request = AiRequest {
            prompt: "OCR text: Hello World".to_string(),
            image_path: None,
        };
        let chat_request = client.build_request(&request).unwrap();
        let arr = chat_request.messages[0].content.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["type"], "text");
        assert!(arr[0]["text"].as_str().unwrap().contains("Hello World"));
    }

    #[test]
    fn test_ask_ai_receives_image_with_text() {
        let tmp = std::env::temp_dir().join("pixelens_test_ask_ai.png");
        std::fs::write(&tmp, b"fake png data").unwrap();
        let client = OpenAiClient::new("https://api.openai.com/v1", "test-key", "gpt-4o", true);
        let request = AiRequest {
            prompt: "OCR text: Hello World".to_string(),
            image_path: Some(tmp.to_string_lossy().to_string()),
        };
        let chat_request = client.build_request(&request).unwrap();
        let arr = chat_request.messages[0].content.as_array().unwrap();
        assert_eq!(arr.len(), 2, "Should have both image and text");
        assert_eq!(arr[0]["type"], "image_url");
        assert!(arr[0]["image_url"]["url"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,"));
        assert_eq!(arr[1]["type"], "text");
        assert!(arr[1]["text"].as_str().unwrap().contains("Hello World"));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_text_only_fallback_when_image_missing() {
        let client = OpenAiClient::new("https://api.openai.com/v1", "test-key", "gpt-4o", true);
        let request = AiRequest {
            prompt: "Describe this".to_string(),
            image_path: Some("/tmp/nonexistent_abc123.png".to_string()),
        };
        let chat_request = client.build_request(&request).unwrap();
        let arr = chat_request.messages[0].content.as_array().unwrap();
        assert_eq!(arr.len(), 1, "Should fall back to text only");
        assert_eq!(arr[0]["type"], "text");
    }

    /// Live end-to-end check against a real OpenAI-compatible provider.
    /// Ignored by default (needs network + a running model host). Run manually:
    ///   PIXELENS_LIVE_AI_ENDPOINT=http://10.0.0.88:11434/v1 \
    ///   PIXELENS_LIVE_AI_MODEL=hermes-qwen3:latest \
    ///   cargo test -p pixelens-ai -- --ignored test_live_ollama_chat
    #[test]
    #[ignore]
    fn test_live_ollama_chat() {
        let endpoint = std::env::var("PIXELENS_LIVE_AI_ENDPOINT")
            .unwrap_or_else(|_| "http://10.0.0.88:11434/v1".to_string());
        let model =
            std::env::var("PIXELENS_LIVE_AI_MODEL").unwrap_or_else(|_| "qwen3.5:9b".to_string());
        // require_key=false: local Ollama/llava tolerates an empty key.
        let client = OpenAiClient::new(&endpoint, "", &model, false);
        let request = AiRequest {
            prompt: "Reply with exactly the words: LIVE AI OK".to_string(),
            image_path: None,
        };
        let result = client.chat(&request);
        assert!(
            result.is_ok(),
            "live Ollama chat failed: {:?}",
            result.err()
        );
        let resp = result.unwrap();
        assert!(
            !resp.content.trim().is_empty(),
            "live Ollama returned empty content"
        );
        println!("LIVE_AI_MODEL={} RESPONSE={:?}", model, resp.content);
    }

    /// Live vision check: sends a real PNG to a vision-capable model and expects
    /// non-empty content back. Ignored by default. Run manually:
    ///   PIXELENS_LIVE_AI_ENDPOINT=http://10.0.0.88:11434/v1 \
    ///   PIXELENS_LIVE_AI_MODEL=hermes-qwen3:latest \
    ///   cargo test -p pixelens-ai -- --ignored test_live_ollama_vision
    #[test]
    #[ignore]
    fn test_live_ollama_vision() {
        let endpoint = std::env::var("PIXELENS_LIVE_AI_ENDPOINT")
            .unwrap_or_else(|_| "http://10.0.0.88:11434/v1".to_string());
        let model =
            std::env::var("PIXELENS_LIVE_AI_MODEL").unwrap_or_else(|_| "qwen3.5:9b".to_string());
        let client = OpenAiClient::new(&endpoint, "", &model, false);
        // 4x4 red PNG (valid; Ollama's loader rejects 1x1).
        let png_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAQAAAAECAIAAAAmkwkpAAAAEElEQVR4nGP4z8AARwzEcQCukw/x0F8jngAAAABJRU5ErkJggg==";
        let png = base64::engine::general_purpose::STANDARD
            .decode(png_b64)
            .expect("valid base64 png");
        let tmp = std::env::temp_dir().join("pixelens_live_vision.png");
        std::fs::write(&tmp, &png).expect("write temp png");
        let request = AiRequest {
            prompt: "What color is this image? Reply in one word.".to_string(),
            image_path: Some(tmp.to_string_lossy().to_string()),
        };
        let result = client.chat(&request);
        let _ = std::fs::remove_file(&tmp);
        assert!(
            result.is_ok(),
            "live Ollama vision call failed: {:?}",
            result.err()
        );
        let resp = result.unwrap();
        assert!(
            !resp.content.trim().is_empty(),
            "live Ollama vision returned empty content"
        );
        println!("LIVE_VISION_MODEL={} RESPONSE={:?}", model, resp.content);
    }
}
