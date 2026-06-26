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
        let entries = "Copy\nSearch\nAsk AI\nTranslate\nCancel";

        let mut child = Command::new("wofi")
            .args(["--dmenu", "-p", "Action: "])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
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
            "Copy" => Ok(MenuChoice::Copy),
            "Search" => Ok(MenuChoice::Search),
            "Ask AI" => Ok(MenuChoice::Ai),
            "Translate" => Ok(MenuChoice::Translate),
            "Cancel" => Ok(MenuChoice::Cancel),
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
}
