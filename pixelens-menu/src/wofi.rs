//! `wofi` launcher backend (a Wayland `dmenu` replacement, like `fuzzel`).
//!
//! Feeds the action list to `wofi --dmenu` over stdin and reads the chosen
//! line from stdout. Non-zero exit (Escape / dismiss) maps to `Cancel`.

use std::io::Write;
use std::process::{Command, Stdio};

use crate::types::{MenuBackend, MenuChoice, MenuError};

/// The newline-delimited entry list shown to the user. The `[X]` prefix is the
/// key hint; the selected line is matched verbatim in [`parse_selection`].
pub const ENTRIES: &str = "[C] Copy\n[S] Search\n[A] Ask AI\n[T] Translate\n[Esc] Cancel";

/// Map a selected line (verbatim stdout from wofi) to a [`MenuChoice`].
///
/// Anything that is not an exact known entry — including an empty line or a
/// dismiss — resolves to `Cancel`, so an Escape never triggers an action.
pub fn parse_selection(line: &str) -> MenuChoice {
    match line.trim() {
        "[C] Copy" => MenuChoice::Copy,
        "[S] Search" => MenuChoice::Search,
        "[A] Ask AI" => MenuChoice::Ai,
        "[T] Translate" => MenuChoice::Translate,
        "[Esc] Cancel" => MenuChoice::Cancel,
        _ => MenuChoice::Cancel,
    }
}

/// True if `wofi` is resolvable on `PATH`.
pub fn is_available() -> bool {
    Command::new("which")
        .arg("wofi")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `wofi` [`MenuBackend`].
pub struct WofiMenu;

impl MenuBackend for WofiMenu {
    fn show_menu(&self, _ocr_text: &str) -> Result<MenuChoice, MenuError> {
        let mut child = Command::new("wofi")
            .args(["--dmenu", "-p", "Action: "])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| MenuError::Backend(format!("failed to run wofi: {e}")))?;

        child
            .stdin
            .as_mut()
            .ok_or_else(|| MenuError::Backend("wofi stdin unavailable".into()))?
            .write_all(ENTRIES.as_bytes())
            .map_err(|e| MenuError::Backend(format!("failed to write to wofi: {e}")))?;

        let output = child
            .wait_with_output()
            .map_err(|e| MenuError::Backend(format!("wofi failed: {e}")))?;

        if !output.status.success() {
            return Ok(MenuChoice::Cancel);
        }

        Ok(parse_selection(&String::from_utf8_lossy(&output.stdout)))
    }

    fn name(&self) -> &str {
        "wofi"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_wofi() {
        assert_eq!(WofiMenu.name(), "wofi");
    }

    #[test]
    fn parse_selection_maps_known_entries() {
        assert_eq!(parse_selection("[C] Copy"), MenuChoice::Copy);
        assert_eq!(parse_selection("[S] Search"), MenuChoice::Search);
        assert_eq!(parse_selection("[A] Ask AI"), MenuChoice::Ai);
        assert_eq!(parse_selection("[T] Translate"), MenuChoice::Translate);
        assert_eq!(parse_selection("[Esc] Cancel"), MenuChoice::Cancel);
    }

    #[test]
    fn parse_selection_falls_back_to_cancel() {
        assert_eq!(parse_selection(""), MenuChoice::Cancel);
        assert_eq!(parse_selection("Copy"), MenuChoice::Cancel);
        assert_eq!(parse_selection("Random text"), MenuChoice::Cancel);
        assert_eq!(parse_selection("  [C] Copy  "), MenuChoice::Copy);
    }

    #[test]
    fn show_menu_reads_shimmed_wofi() {
        // Shim `wofi` on PATH: echo the requested entry and exit 0.
        let dir = std::env::temp_dir().join(format!("pixelens-menu-shim-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("wofi");
        std::fs::write(
            &bin,
            "#!/bin/sh\ncat >/dev/null\nprintf '%s' \"$PIXELENS_SHIM_OUTPUT\"\n",
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&bin).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&bin, perms).unwrap();
        }

        let prev = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{}:{}", dir.to_string_lossy(), prev));
        std::env::set_var("PIXELENS_SHIM_OUTPUT", "[A] Ask AI");

        let result = WofiMenu.show_menu("ocr text").unwrap();
        assert_eq!(result, MenuChoice::Ai);

        std::env::set_var("PIXELENS_SHIM_OUTPUT", "[T] Translate");
        let result = WofiMenu.show_menu("ocr text").unwrap();
        assert_eq!(result, MenuChoice::Translate);

        std::env::remove_var("PIXELENS_SHIM_OUTPUT");
        std::env::set_var("PATH", prev);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
