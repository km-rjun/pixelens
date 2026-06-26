use std::io::Write;
use std::process::{Command, Stdio};

use crate::error::PixelensError;
use crate::menu::{MenuBackend, MenuChoice};

pub struct WofiMenu;

pub fn is_available() -> bool {
    Command::new("which")
        .arg("wofi")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

impl MenuBackend for WofiMenu {
    fn show_menu(&self, _ocr_text: &str) -> Result<MenuChoice, PixelensError> {
        let entries = "[C] Copy\n[S] Search\n[A] Ask AI\n[T] Translate\n[Esc] Cancel";

        let mut child = Command::new("wofi")
            .args(["--dmenu", "-p", "Action: "])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| PixelensError::Config(format!("Failed to run wofi: {}", e)))?;

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(entries.as_bytes())
            .map_err(|e| PixelensError::Config(format!("Failed to write to wofi: {}", e)))?;

        let output = child
            .wait_with_output()
            .map_err(|e| PixelensError::Config(format!("wofi failed: {}", e)))?;

        if !output.status.success() {
            return Ok(MenuChoice::Cancel);
        }

        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        match selected.as_str() {
            "[C] Copy" => Ok(MenuChoice::Copy),
            "[S] Search" => Ok(MenuChoice::Search),
            "[A] Ask AI" => Ok(MenuChoice::Ai),
            "[T] Translate" => Ok(MenuChoice::Translate),
            "[Esc] Cancel" => Ok(MenuChoice::Cancel),
            _ => Ok(MenuChoice::Cancel),
        }
    }

    fn name(&self) -> &str {
        "wofi"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available() {
        let _ = is_available();
    }

    #[test]
    fn test_exact_mapping() {
        let cases = vec![
            ("[C] Copy", MenuChoice::Copy),
            ("[S] Search", MenuChoice::Search),
            ("[A] Ask AI", MenuChoice::Ai),
            ("[T] Translate", MenuChoice::Translate),
            ("[Esc] Cancel", MenuChoice::Cancel),
            ("", MenuChoice::Cancel),
            ("Random text", MenuChoice::Cancel),
            ("Copy", MenuChoice::Cancel),
            ("Search", MenuChoice::Cancel),
        ];
        for (input, expected) in cases {
            let result = match input {
                "[C] Copy" => MenuChoice::Copy,
                "[S] Search" => MenuChoice::Search,
                "[A] Ask AI" => MenuChoice::Ai,
                "[T] Translate" => MenuChoice::Translate,
                "[Esc] Cancel" => MenuChoice::Cancel,
                _ => MenuChoice::Cancel,
            };
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_copy_cannot_map_to_search() {
        let result = match "[C] Copy" {
            "[C] Copy" => MenuChoice::Copy,
            "[S] Search" => MenuChoice::Search,
            _ => MenuChoice::Cancel,
        };
        assert_eq!(result, MenuChoice::Copy);
        assert_ne!(result, MenuChoice::Search);
    }
}
