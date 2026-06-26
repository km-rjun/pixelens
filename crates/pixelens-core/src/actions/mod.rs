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

pub fn available_actions() -> Vec<(&'static str, &'static str)> {
    vec![
        ("1", "copy"),
        ("2", "search"),
        ("3", "translate"),
        ("4", "ai"),
    ]
}

pub fn parse_action_choice(choice: &str, text: &str) -> Option<ActionType> {
    match choice.trim() {
        "1" | "copy" => Some(ActionType::CopyToClipboard),
        "2" | "search" => Some(ActionType::SearchWeb),
        "3" | "translate" => Some(ActionType::Translate("English".to_string())),
        "4" | "ai" => Some(ActionType::AskAi(text.to_string())),
        _ => None,
    }
}

pub fn print_action_menu() {
    eprintln!("\nAvailable actions:");
    eprintln!("  1) copy    - Copy text to clipboard");
    eprintln!("  2) search  - Search the web");
    eprintln!("  3) translate - Translate text");
    eprintln!("  4) ai      - Ask AI");
    eprintln!("\nEnter choice (1-4) or 'q' to quit:");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_action_choice_copy() {
        assert!(matches!(
            parse_action_choice("1", "text"),
            Some(ActionType::CopyToClipboard)
        ));
        assert!(matches!(
            parse_action_choice("copy", "text"),
            Some(ActionType::CopyToClipboard)
        ));
    }

    #[test]
    fn test_parse_action_choice_search() {
        assert!(matches!(
            parse_action_choice("2", "text"),
            Some(ActionType::SearchWeb)
        ));
    }

    #[test]
    fn test_parse_action_choice_invalid() {
        assert!(parse_action_choice("5", "text").is_none());
        assert!(parse_action_choice("q", "text").is_none());
    }

    #[test]
    fn test_available_actions_count() {
        assert_eq!(available_actions().len(), 4);
    }
}
