use std::io::{self, BufRead};

use crate::error::PixelensError;
use crate::menu::{MenuBackend, MenuChoice};

pub struct StdinMenu;

impl MenuBackend for StdinMenu {
    fn show_menu(&self, _ocr_text: &str) -> Result<MenuChoice, PixelensError> {
        eprintln!("\nActions: [C]opy  [S]earch  [A]sk AI  [T]ranslate  [Esc] Cancel");
        eprint!("> ");

        let stdin = io::stdin();
        let mut line = String::new();
        stdin
            .lock()
            .read_line(&mut line)
            .map_err(|e| PixelensError::Config(format!("Failed to read input: {}", e)))?;

        let key = line.trim();
        MenuChoice::from_key(key)
            .ok_or_else(|| PixelensError::Config(format!("Invalid choice: {}", key)))
    }

    fn name(&self) -> &str {
        "stdin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let menu = StdinMenu;
        assert_eq!(menu.name(), "stdin");
    }
}
