use super::ActionHandler;
use crate::error::PixelensError;
use crate::types::{ActionPayload, ActionType};

pub struct TranslateHandler {
    pub target_lang: String,
}

impl ActionHandler for TranslateHandler {
    fn execute(&self, payload: &ActionPayload) -> Result<String, PixelensError> {
        let prompt = format!(
            "Translate the following text to {}. Return only the translation:\n\n{}",
            self.target_lang, payload.text
        );
        Ok(prompt)
    }

    fn action_type(&self) -> ActionType {
        ActionType::Translate(self.target_lang.clone())
    }
}
