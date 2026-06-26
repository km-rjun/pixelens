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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_handler() {
        let handler = TranslateHandler {
            target_lang: "Spanish".to_string(),
        };
        let payload = ActionPayload {
            text: "Hello".to_string(),
            image_path: None,
        };
        let result = handler.execute(&payload).unwrap();
        assert!(result.contains("Spanish"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_translate_handler_different_lang() {
        let handler = TranslateHandler {
            target_lang: "Japanese".to_string(),
        };
        let payload = ActionPayload {
            text: "Good morning".to_string(),
            image_path: None,
        };
        let result = handler.execute(&payload).unwrap();
        assert!(result.contains("Japanese"));
        assert!(result.contains("Good morning"));
    }

    #[test]
    fn test_action_type() {
        let handler = TranslateHandler {
            target_lang: "French".to_string(),
        };
        assert!(matches!(handler.action_type(), ActionType::Translate(_)));
    }
}
