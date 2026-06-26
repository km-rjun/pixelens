use std::process::Command;

use crate::actions::ActionHandler;
use crate::error::PixelensError;
use crate::types::{ActionPayload, ActionType};

pub fn build_search_url(text: &str) -> String {
    let encoded = urlencoding::encode(text);
    format!("https://www.google.com/search?q={}", encoded)
}

pub trait UrlLauncher {
    fn open(&self, url: &str) -> Result<(), String>;
}

pub struct XdgLauncher;

impl UrlLauncher for XdgLauncher {
    fn open(&self, url: &str) -> Result<(), String> {
        let status = Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|e| format!("Failed to run xdg-open: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("xdg-open exited with status: {}", status))
        }
    }
}

pub struct SearchHandler<L: UrlLauncher = XdgLauncher> {
    launcher: L,
}

impl<L: UrlLauncher> SearchHandler<L> {
    pub fn new(launcher: L) -> Self {
        Self { launcher }
    }
}

impl Default for SearchHandler {
    fn default() -> Self {
        Self {
            launcher: XdgLauncher,
        }
    }
}

impl<L: UrlLauncher> ActionHandler for SearchHandler<L> {
    fn execute(&self, payload: &ActionPayload) -> Result<String, PixelensError> {
        let url = build_search_url(&payload.text);
        self.launcher
            .open(&url)
            .map_err(|e| PixelensError::Config(format!("{}\nURL: {}", e, url)))?;
        Ok(String::new())
    }

    fn action_type(&self) -> ActionType {
        ActionType::SearchWeb
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockLauncher {
        last_url: RefCell<Option<String>>,
        should_fail: bool,
    }

    impl MockLauncher {
        fn new(should_fail: bool) -> Self {
            Self {
                last_url: RefCell::new(None),
                should_fail,
            }
        }
    }

    impl UrlLauncher for MockLauncher {
        fn open(&self, url: &str) -> Result<(), String> {
            *self.last_url.borrow_mut() = Some(url.to_string());
            if self.should_fail {
                Err("mock launcher failed".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_build_search_url_simple() {
        let url = build_search_url("rust programming");
        assert_eq!(url, "https://www.google.com/search?q=rust%20programming");
    }

    #[test]
    fn test_build_search_url_special_chars() {
        let url = build_search_url("hello & world");
        assert!(url.contains("google.com"));
        assert!(url.contains("hello"));
        assert!(url.contains("world"));
        assert!(url.contains("%26"));
    }

    #[test]
    fn test_build_search_url_multiline() {
        let url = build_search_url("line1\nline2");
        assert!(url.contains("line1"));
        assert!(url.contains("line2"));
    }

    #[test]
    fn test_search_opener_success() {
        let launcher = MockLauncher::new(false);
        let handler = SearchHandler::new(launcher);
        let payload = ActionPayload {
            text: "rust programming".to_string(),
            image_path: None,
        };
        let result = handler.execute(&payload);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_opener_failure() {
        let launcher = MockLauncher::new(true);
        let handler = SearchHandler::new(launcher);
        let payload = ActionPayload {
            text: "rust programming".to_string(),
            image_path: None,
        };
        let result = handler.execute(&payload);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("mock launcher failed"));
        assert!(err.contains("google.com"));
    }

    #[test]
    fn test_search_action_type() {
        let handler = SearchHandler::default();
        assert!(matches!(handler.action_type(), ActionType::SearchWeb));
    }
}
