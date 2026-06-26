pub mod ai;
pub mod copy;
pub mod reverse_image;
pub mod search;
pub mod translate;

use crate::error::PixelensError;
use crate::types::{ActionPayload, ActionType};

pub trait ActionHandler {
    fn execute(&self, payload: &ActionPayload) -> Result<String, PixelensError>;
    fn action_type(&self) -> ActionType;
}

pub fn get_handler(action: &ActionType) -> Result<Box<dyn ActionHandler>, PixelensError> {
    match action {
        ActionType::CopyToClipboard => Ok(Box::new(copy::CopyHandler)),
        ActionType::SearchWeb => Ok(Box::new(search::SearchHandler)),
        ActionType::ReverseImageSearch => Ok(Box::new(reverse_image::ReverseImageHandler)),
        ActionType::Translate(ref lang) => Ok(Box::new(translate::TranslateHandler {
            target_lang: lang.clone(),
        })),
        ActionType::AskAi(_) => Err(PixelensError::Config(
            "AI actions should be handled via the AI command".to_string(),
        )),
    }
}

pub fn get_action_names() -> Vec<String> {
    vec![
        "copy".to_string(),
        "search".to_string(),
        "reverse_image".to_string(),
        "translate".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_handler_copy() {
        let handler = get_handler(&ActionType::CopyToClipboard).unwrap();
        assert!(matches!(handler.action_type(), ActionType::CopyToClipboard));
    }

    #[test]
    fn test_get_handler_search() {
        let handler = get_handler(&ActionType::SearchWeb).unwrap();
        assert!(matches!(handler.action_type(), ActionType::SearchWeb));
    }

    #[test]
    fn test_get_handler_reverse_image() {
        let handler = get_handler(&ActionType::ReverseImageSearch).unwrap();
        assert!(matches!(
            handler.action_type(),
            ActionType::ReverseImageSearch
        ));
    }

    #[test]
    fn test_get_handler_translate() {
        let handler = get_handler(&ActionType::Translate("German".to_string())).unwrap();
        assert!(matches!(handler.action_type(), ActionType::Translate(_)));
    }

    #[test]
    fn test_get_handler_ask_ai() {
        let result = get_handler(&ActionType::AskAi("test".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_action_names() {
        let names = get_action_names();
        assert!(names.contains(&"copy".to_string()));
        assert!(names.contains(&"search".to_string()));
        assert!(names.contains(&"reverse_image".to_string()));
        assert!(names.contains(&"translate".to_string()));
    }
}
