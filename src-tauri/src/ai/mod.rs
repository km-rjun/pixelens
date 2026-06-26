pub mod client;

use crate::error::AiError;
use crate::types::{AiRequest, AiResponse};

pub trait AiProvider {
    fn chat(&self, request: &AiRequest) -> Result<AiResponse, AiError>;
    fn name(&self) -> &str;
}

pub fn create_provider(endpoint: &str, api_key: &str, model: &str) -> Box<dyn AiProvider> {
    Box::new(client::OpenAiClient::new(
        endpoint.to_string(),
        api_key.to_string(),
        model.to_string(),
    ))
}

pub fn check_api_reachable(endpoint: &str) -> bool {
    let url = format!("{}/models", endpoint.trim_end_matches('/'));
    ureq::get(&url).call().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_provider() {
        let provider = create_provider("https://api.openai.com/v1", "test-key", "gpt-4o");
        assert_eq!(provider.name(), "openai");
    }
}
