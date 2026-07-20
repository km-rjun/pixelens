//! Headless menu backend: reads a single keystroke from stdin.
//!
//! Used by default in environments without a graphical launcher (CI, SSH,
//! testing) and as the final fallback in [`detect_backend`](crate::detect_backend).

use std::io::{self, BufRead};

use crate::types::{MenuBackend, MenuChoice, MenuError};

/// Parse a raw stdin line into a [`MenuChoice`].
///
/// Separated from [`StdinMenu::show_menu`] so the mapping is unit-testable
/// without redirecting stdin.
pub fn parse_choice(line: &str) -> Result<MenuChoice, MenuError> {
    let key = line.trim();
    MenuChoice::from_key(key).ok_or_else(|| MenuError::Other(format!("invalid choice: {key}")))
}

/// Headless [`MenuBackend`] that prompts on stderr and reads the choice from
/// stdin.
pub struct StdinMenu;

impl MenuBackend for StdinMenu {
    fn show_menu(&self, _ocr_text: &str) -> Result<MenuChoice, MenuError> {
        eprintln!("\nActions: [C]opy  [S]earch  [A]sk AI  [T]ranslate  [Esc] Cancel");
        eprint!("> ");

        let mut line = String::new();
        io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(MenuError::Io)?;

        parse_choice(&line)
    }

    fn name(&self) -> &str {
        "stdin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_stdin() {
        assert_eq!(StdinMenu.name(), "stdin");
    }

    #[test]
    fn parse_choice_maps_keys() {
        assert_eq!(parse_choice("c\n").unwrap(), MenuChoice::Copy);
        assert_eq!(parse_choice("S").unwrap(), MenuChoice::Search);
        assert_eq!(parse_choice(" a ").unwrap(), MenuChoice::Ai);
        assert_eq!(parse_choice("t").unwrap(), MenuChoice::Translate);
        assert_eq!(parse_choice("").unwrap(), MenuChoice::Cancel);
        assert_eq!(parse_choice("esc").unwrap(), MenuChoice::Cancel);
    }

    #[test]
    fn parse_choice_rejects_unknown() {
        let err = parse_choice("x").expect_err("unknown key must error");
        assert!(err.to_string().contains("invalid choice"), "got: {err}");
        let err2 = parse_choice("5").expect_err("unknown key must error");
        assert!(err2.to_string().contains("invalid choice"), "got: {err2}");
    }
}
