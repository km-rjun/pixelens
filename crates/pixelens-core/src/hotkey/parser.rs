use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Hotkey {
    pub modifiers: HashSet<String>,
    pub key: String,
}

#[derive(Debug, thiserror::Error)]
pub enum HotkeyError {
    #[error("Invalid hotkey format: {0}")]
    InvalidFormat(String),

    #[error("Unknown key: {0}")]
    UnknownKey(String),
}

const VALID_KEYS: &[&str] = &[
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "space",
    "enter",
    "return",
    "escape",
    "esc",
    "tab",
    "backspace",
    "delete",
    "del",
    "up",
    "down",
    "left",
    "right",
    "home",
    "end",
    "pageup",
    "pagedown",
    "f1",
    "f2",
    "f3",
    "f4",
    "f5",
    "f6",
    "f7",
    "f8",
    "f9",
    "f10",
    "f11",
    "f12",
];

impl FromStr for Hotkey {
    type Err = HotkeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            return Err(HotkeyError::InvalidFormat("Empty hotkey".to_string()));
        }

        let parts: Vec<&str> = s
            .split('+')
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();

        if parts.is_empty() {
            return Err(HotkeyError::InvalidFormat("Empty hotkey".to_string()));
        }

        let mut modifiers = HashSet::new();
        let mut key = None;

        for part in parts {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => {
                    modifiers.insert("ctrl".to_string());
                }
                "shift" => {
                    modifiers.insert("shift".to_string());
                }
                "alt" => {
                    modifiers.insert("alt".to_string());
                }
                "super" | "meta" | "win" => {
                    modifiers.insert("super".to_string());
                }
                k if VALID_KEYS.contains(&k) => {
                    if key.is_some() {
                        return Err(HotkeyError::InvalidFormat(
                            "Multiple non-modifier keys".to_string(),
                        ));
                    }
                    key = Some(k.to_string());
                }
                _ => {
                    return Err(HotkeyError::UnknownKey(part.to_string()));
                }
            }
        }

        let key = key.ok_or_else(|| HotkeyError::InvalidFormat("No key specified".to_string()))?;

        Ok(Self { modifiers, key })
    }
}

impl Hotkey {
    pub fn matches_string(&self, modifiers: &[&str], key: &str) -> bool {
        let pressed_modifiers: HashSet<String> =
            modifiers.iter().map(|s| s.to_lowercase()).collect();
        self.modifiers.iter().all(|m| pressed_modifiers.contains(m))
            && self.key == key.to_lowercase()
    }
}

impl std::fmt::Display for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts: Vec<String> = self.modifiers.iter().cloned().collect();
        parts.sort();
        parts.push(self.key.clone());
        write!(f, "{}", parts.join("+"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ctrl_c() {
        let hotkey: Hotkey = "Ctrl+C".parse().unwrap();
        assert!(hotkey.modifiers.contains("ctrl"));
        assert_eq!(hotkey.key, "c");
    }

    #[test]
    fn test_parse_ctrl_shift_c() {
        let hotkey: Hotkey = "Ctrl+Shift+C".parse().unwrap();
        assert!(hotkey.modifiers.contains("ctrl"));
        assert!(hotkey.modifiers.contains("shift"));
        assert_eq!(hotkey.key, "c");
    }

    #[test]
    fn test_parse_super_space() {
        let hotkey: Hotkey = "Super+Space".parse().unwrap();
        assert!(hotkey.modifiers.contains("super"));
        assert_eq!(hotkey.key, "space");
    }

    #[test]
    fn test_parse_invalid_empty() {
        assert!("".parse::<Hotkey>().is_err());
    }

    #[test]
    fn test_parse_invalid_whitespace() {
        assert!("   ".parse::<Hotkey>().is_err());
    }

    #[test]
    fn test_parse_invalid_unknown_key() {
        assert!("Ctrl+Unknown".parse::<Hotkey>().is_err());
    }

    #[test]
    fn test_matches() {
        let hotkey: Hotkey = "Ctrl+C".parse().unwrap();
        assert!(hotkey.matches_string(&["ctrl"], "c"));
    }

    #[test]
    fn test_matches_missing_modifier() {
        let hotkey: Hotkey = "Ctrl+Shift+C".parse().unwrap();
        assert!(!hotkey.matches_string(&["ctrl"], "c"));
    }

    #[test]
    fn test_display() {
        let hotkey: Hotkey = "Ctrl+Shift+C".parse().unwrap();
        assert_eq!(hotkey.to_string(), "ctrl+shift+c");
    }
}
