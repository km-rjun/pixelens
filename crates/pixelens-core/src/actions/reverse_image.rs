use crate::actions::ActionHandler;
use crate::error::PixelensError;
use crate::types::{ActionPayload, ActionType};

pub struct ReverseImageHandler;

impl ActionHandler for ReverseImageHandler {
    fn execute(&self, _payload: &ActionPayload) -> Result<String, PixelensError> {
        Err(PixelensError::Config(
            "Reverse image search is not yet implemented".to_string(),
        ))
    }

    fn action_type(&self) -> ActionType {
        ActionType::ReverseImageSearch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_image_handler_not_implemented() {
        let handler = ReverseImageHandler;
        let payload = ActionPayload {
            text: String::new(),
            image_path: Some("/tmp/screenshot.png".to_string()),
        };
        let result = handler.execute(&payload);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not yet implemented"));
    }

    #[test]
    fn test_action_type() {
        let handler = ReverseImageHandler;
        assert!(matches!(
            handler.action_type(),
            ActionType::ReverseImageSearch
        ));
    }
}
