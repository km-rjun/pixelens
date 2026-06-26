use crate::ActionHandler;
use pixelens_common::{ActionPayload, ActionType, PixelensError};

pub struct CopyHandler;

impl ActionHandler for CopyHandler {
    fn execute(&self, payload: &ActionPayload) -> Result<String, PixelensError> {
        Ok(payload.text.clone())
    }

    fn action_type(&self) -> ActionType {
        ActionType::CopyToClipboard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_handler() {
        let handler = CopyHandler;
        let payload = ActionPayload {
            text: "Hello World".to_string(),
            image_path: None,
        };
        let result = handler.execute(&payload).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_copy_handler_empty() {
        let handler = CopyHandler;
        let payload = ActionPayload {
            text: String::new(),
            image_path: None,
        };
        let result = handler.execute(&payload).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_action_type() {
        let handler = CopyHandler;
        assert!(matches!(handler.action_type(), ActionType::CopyToClipboard));
    }
}
