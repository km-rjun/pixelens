use super::ActionHandler;
use crate::error::PixelensError;
use crate::types::{ActionPayload, ActionType};

pub struct SearchHandler;

impl ActionHandler for SearchHandler {
    fn execute(&self, payload: &ActionPayload) -> Result<String, PixelensError> {
        let encoded = urlencoding::encode(&payload.text);
        let url = format!("https://www.google.com/search?q={}", encoded);
        Ok(url)
    }

    fn action_type(&self) -> ActionType {
        ActionType::SearchWeb
    }
}
