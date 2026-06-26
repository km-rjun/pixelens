pub mod parser;

pub use parser::{Hotkey, HotkeyError};

use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct HotkeyConfig {
    pub hotkey: Hotkey,
}

impl HotkeyConfig {
    pub fn new(hotkey: Hotkey) -> Self {
        Self { hotkey }
    }

    pub fn parse(s: &str) -> Result<Self, HotkeyError> {
        let hotkey = Hotkey::from_str(s)?;
        Ok(Self { hotkey })
    }

    pub fn matches(&self, modifiers: &[&str], key: &str) -> bool {
        self.hotkey.matches_string(modifiers, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_config() {
        let config = HotkeyConfig::parse("Ctrl+Shift+C").unwrap();
        assert!(config.matches(&["ctrl", "shift"], "c"));
    }
}
