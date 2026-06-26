use super::ActionHandler;
use crate::error::PixelensError;
use crate::types::{ActionPayload, ActionType};

pub struct ReverseImageHandler;

impl ActionHandler for ReverseImageHandler {
    fn execute(&self, payload: &ActionPayload) -> Result<String, PixelensError> {
        let image_path = payload.image_path.as_ref().ok_or_else(|| {
            PixelensError::Config("No image provided for reverse search".to_string())
        })?;

        let url = format!(
            "https://lens.google.com/uploadbyurl?url=file://{}",
            urlencoding::encode(image_path)
        );
        Ok(url)
    }

    fn action_type(&self) -> ActionType {
        ActionType::ReverseImageSearch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_image_handler() {
        let handler = ReverseImageHandler;
        let payload = ActionPayload {
            text: String::new(),
            image_path: Some("/tmp/screenshot.png".to_string()),
        };
        let result = handler.execute(&payload).unwrap();
        assert!(result.contains("lens.google.com"));
        assert!(result.contains("screenshot.png"));
    }

    #[test]
    fn test_reverse_image_handler_no_image() {
        let handler = ReverseImageHandler;
        let payload = ActionPayload {
            text: String::new(),
            image_path: None,
        };
        let result = handler.execute(&payload);
        assert!(result.is_err());
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
