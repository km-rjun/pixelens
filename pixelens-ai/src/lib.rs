//! `pixelens-ai` — OpenAI-compatible AI client for Pixelens.
//!
//! Ported from `origin/main:crates/pixelens-core/src/ai` (Strategy C, 2026-07-20).
//! Key adaptation: the HTTP layer uses `reqwest::blocking` so `OpenAiClient::chat()`
//! stays synchronous, letting the daemon invoke it via `tokio::task::spawn_blocking`.
//!
//! The client is vision-capable (base64 image embedding for vision models) and
//! defaults to a local Ollama endpoint (`http://10.0.0.1:11434/v1`) which needs no
//! API key (`AiConfig.require_key = false`).

mod client;
mod error;
mod provider_error;
mod types;

pub use client::{model_supports_vision, OpenAiClient};
pub use error::{AiError, RateLimitKind};
pub use provider_error::{parse_429_response, parse_retry_after};
pub use types::{AiRequest, AiResponse};

#[cfg(test)]
mod live {
    use super::*;
    use std::io::Write;

    /// Live smoke test against a real Ollama server.
    ///
    /// Run with: `cargo test -p pixelens-ai -- --ignored`
    /// Endpoint/model are taken from config, overridable via env:
    ///   PIXELENS_AI_ENDPOINT, PIXELENS_AI_MODEL, PIXELENS_AI_API_KEY
    #[test]
    #[ignore = "requires a live OpenAI-compatible server (e.g. Ollama)"]
    fn live_ollama_chat() {
        let endpoint = std::env::var("PIXELENS_AI_ENDPOINT")
            .unwrap_or_else(|_| "http://10.0.0.1:11434/v1".to_string());
        let model = std::env::var("PIXELENS_AI_MODEL").unwrap_or_else(|_| "llava".to_string());
        let api_key = std::env::var("PIXELENS_AI_API_KEY").unwrap_or_default();

        let client = OpenAiClient::new(endpoint, api_key, model, false);

        let request = AiRequest {
            prompt: "Reply with the single word: pong".to_string(),
            image_path: None,
        };

        match client.chat(&request) {
            Ok(resp) => {
                let stdout = std::io::stdout();
                let mut lock = stdout.lock();
                writeln!(
                    lock,
                    "live_ollama_chat ok: model={} content={}",
                    resp.model, resp.content
                )
                .unwrap();
                assert!(!resp.content.trim().is_empty(), "empty response from model");
            }
            Err(e) => panic!("live Ollama chat failed: {}", e),
        }
    }
}
