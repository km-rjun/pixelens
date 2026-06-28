use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{self, Write};

use crate::error::PixelensError;
use crate::menu::{MenuBackend, MenuChoice};

pub struct ActionBar;

impl MenuBackend for ActionBar {
    fn show_menu(&self, _ocr_text: &str) -> Result<MenuChoice, PixelensError> {
        enable_raw_mode()
            .map_err(|e| PixelensError::Config(format!("Failed to enable raw mode: {}", e)))?;

        let result = self.run_selection();

        disable_raw_mode().ok();

        result
    }

    fn name(&self) -> &str {
        "action_bar"
    }
}

impl ActionBar {
    fn run_selection(&self) -> Result<MenuChoice, PixelensError> {
        let mut stdout = io::stdout();
        execute!(stdout, Clear(ClearType::All))
            .map_err(|e| PixelensError::Config(format!("Failed to clear screen: {}", e)))?;

        println!("╔══════════════════════════════════════════════════════╗");
        println!("║                  Pixelens Action Bar                 ║");
        println!("╠══════════════════════════════════════════════════════╣");
        println!("║  [C] Copy    [S] Search    [A] Ask AI    [T] Translate ║");
        println!("║                      [Esc] Cancel                   ║");
        println!("╚══════════════════════════════════════════════════════╝");
        println!();
        print!("Select action: ");
        stdout.flush().ok();

        loop {
            if event::poll(std::time::Duration::from_millis(100))
                .map_err(|e| PixelensError::Config(format!("Event poll failed: {}", e)))?
            {
                if let Event::Key(KeyEvent { code, .. }) = event::read()
                    .map_err(|e| PixelensError::Config(format!("Failed to read key: {}", e)))?
                {
                    match code {
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            return Ok(MenuChoice::Copy);
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            return Ok(MenuChoice::Search);
                        }
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            return Ok(MenuChoice::Ai);
                        }
                        KeyCode::Char('t') | KeyCode::Char('T') => {
                            return Ok(MenuChoice::Translate);
                        }
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                            return Ok(MenuChoice::Cancel);
                        }
                        KeyCode::Char('x') | KeyCode::Char('X') => {
                            return Ok(MenuChoice::Cancel);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_bar_name() {
        let menu = ActionBar;
        assert_eq!(menu.name(), "action_bar");
    }
}
