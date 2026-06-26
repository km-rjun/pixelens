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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_handler() {
        let handler = SearchHandler;
        let payload = ActionPayload {
            text: "rust programming".to_string(),
            image_path: None,
        };
        let result = handler.execute(&payload).unwrap();
        assert!(result.contains("google.com"));
        assert!(result.contains("rust"));
        assert!(result.contains("programming"));
    }

    #[test]
    fn test_search_handler_special_chars() {
        let handler = SearchHandler;
        let payload = ActionPayload {
            text: "hello & world".to_string(),
            image_path: None,
        };
        let result = handler.execute(&payload).unwrap();
        assert!(result.contains("google.com"));
        assert!(result.contains("hello"));
        assert!(result.contains("world"));
    }

    #[test]
    fn test_search_handler_empty() {
        let handler = SearchHandler;
        let payload = ActionPayload {
            text: String::new(),
            image_path: None,
        };
        let result = handler.execute(&payload).unwrap();
        assert!(result.contains("google.com"));
    }

    #[test]
    fn test_action_type() {
        let handler = SearchHandler;
        assert!(matches!(handler.action_type(), ActionType::SearchWeb));
    }
}
