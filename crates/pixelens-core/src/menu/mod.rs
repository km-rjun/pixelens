pub mod action_bar;
pub mod fuzzel;
pub mod stdin;
pub mod wofi;

use crate::error::PixelensError;

#[derive(Debug, Clone, PartialEq)]
pub enum MenuChoice {
    Copy,
    Search,
    Ai,
    Translate,
    Cancel,
}

impl MenuChoice {
    pub fn from_key(key: &str) -> Option<Self> {
        match key.trim().to_lowercase().as_str() {
            "c" => Some(MenuChoice::Copy),
            "s" => Some(MenuChoice::Search),
            "a" => Some(MenuChoice::Ai),
            "t" => Some(MenuChoice::Translate),
            "escape" | "esc" | "q" | "" => Some(MenuChoice::Cancel),
            _ => None,
        }
    }
}

pub trait MenuBackend {
    fn show_menu(&self, ocr_text: &str) -> Result<MenuChoice, PixelensError>;
    fn name(&self) -> &str;
}

pub fn detect_backend() -> Result<Box<dyn MenuBackend>, PixelensError> {
    Ok(Box::new(action_bar::ActionBar))
}

pub fn create_backend(name: &str) -> Result<Box<dyn MenuBackend>, PixelensError> {
    match name {
        "action_bar" | "built-in" | "auto" => Ok(Box::new(action_bar::ActionBar)),
        "fuzzel" => Ok(Box::new(fuzzel::FuzzelMenu)),
        "wofi" => Ok(Box::new(wofi::WofiMenu)),
        "stdin" => Ok(Box::new(stdin::StdinMenu)),
        _ => Err(PixelensError::Config(format!(
            "Unknown menu backend: {}. Supported: action_bar, fuzzel, wofi, stdin",
            name
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_choice_from_key() {
        assert_eq!(MenuChoice::from_key("c"), Some(MenuChoice::Copy));
        assert_eq!(MenuChoice::from_key("C"), Some(MenuChoice::Copy));
        assert_eq!(MenuChoice::from_key("s"), Some(MenuChoice::Search));
        assert_eq!(MenuChoice::from_key("S"), Some(MenuChoice::Search));
        assert_eq!(MenuChoice::from_key("a"), Some(MenuChoice::Ai));
        assert_eq!(MenuChoice::from_key("A"), Some(MenuChoice::Ai));
        assert_eq!(MenuChoice::from_key("t"), Some(MenuChoice::Translate));
        assert_eq!(MenuChoice::from_key("T"), Some(MenuChoice::Translate));
        assert_eq!(MenuChoice::from_key("escape"), Some(MenuChoice::Cancel));
        assert_eq!(MenuChoice::from_key("esc"), Some(MenuChoice::Cancel));
        assert_eq!(MenuChoice::from_key("q"), Some(MenuChoice::Cancel));
        assert_eq!(MenuChoice::from_key(""), Some(MenuChoice::Cancel));
        assert_eq!(MenuChoice::from_key("x"), None);
        assert_eq!(MenuChoice::from_key("5"), None);
    }

    #[test]
    fn test_create_backend_stdin() {
        let backend = create_backend("stdin").unwrap();
        assert_eq!(backend.name(), "stdin");
    }

    #[test]
    fn test_create_backend_unknown() {
        let result = create_backend("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_never_maps_to_search() {
        let copy_result = MenuChoice::from_key("c");
        let search_result = MenuChoice::from_key("s");
        assert_eq!(copy_result, Some(MenuChoice::Copy));
        assert_eq!(search_result, Some(MenuChoice::Search));
        assert_ne!(copy_result, search_result);
    }
}
