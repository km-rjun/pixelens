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
