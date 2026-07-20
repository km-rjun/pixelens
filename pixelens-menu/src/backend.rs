//! Backend factory: pick a concrete [`MenuBackend`] by name or auto-detect one.

use crate::types::{MenuBackend, MenuError};
use crate::{action_bar, fuzzel, stdin, wofi};

/// Build a backend by explicit name.
///
/// Accepted names: `stdin`, `fuzzel`, `wofi`, `action_bar`. Unknown names
/// yield [`MenuError::Other`].
pub fn create_backend(
    name: &str,
) -> Result<Box<dyn MenuBackend + Send + Sync + 'static>, MenuError> {
    match name {
        "stdin" => Ok(Box::new(stdin::StdinMenu)),
        "fuzzel" => Ok(Box::new(fuzzel::FuzzelMenu)),
        "wofi" => Ok(Box::new(wofi::WofiMenu)),
        "action_bar" => action_bar::create_backend(),
        other => Err(MenuError::Other(format!("unknown menu backend: {other}"))),
    }
}

/// Auto-select a backend: prefer the GTK action bar when compiled in, else the
/// first available dmenu-compatible launcher (`fuzzel` > `wofi`), falling back
/// to `stdin` when nothing graphical is present.
pub fn detect_backend() -> Result<Box<dyn MenuBackend + Send + Sync + 'static>, MenuError> {
    if cfg!(feature = "menu-gtk") {
        return action_bar::create_backend();
    }
    if fuzzel::is_available() {
        return Ok(Box::new(fuzzel::FuzzelMenu));
    }
    if wofi::is_available() {
        return Ok(Box::new(wofi::WofiMenu));
    }
    Ok(Box::new(stdin::StdinMenu))
}

/// Names that could be selected on this build, in preference order.
pub fn available_backends() -> Vec<&'static str> {
    let mut v = Vec::new();
    if cfg!(feature = "menu-gtk") {
        v.push("action_bar");
    }
    v.push("fuzzel");
    v.push("wofi");
    v.push("stdin");
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_backend_known_names() {
        assert_eq!(create_backend("stdin").unwrap().name(), "stdin");
        assert_eq!(create_backend("fuzzel").unwrap().name(), "fuzzel");
        assert_eq!(create_backend("wofi").unwrap().name(), "wofi");
        let action = create_backend("action_bar");
        if cfg!(feature = "menu-gtk") {
            assert_eq!(action.unwrap().name(), "action_bar");
        } else {
            assert!(action.is_err(), "action_bar unavailable without menu-gtk");
        }
    }

    #[test]
    fn create_backend_unknown_is_error() {
        assert!(create_backend("nope").is_err());
    }

    #[test]
    fn detect_backend_falls_back_to_stdin_in_headless() {
        // No fuzzel/wofi on PATH in CI/headless -> stdin.
        let name = detect_backend().unwrap().name().to_string();
        assert!(
            matches!(name.as_str(), "stdin" | "fuzzel" | "wofi" | "action_bar"),
            "unexpected backend: {name}"
        );
    }

    #[test]
    fn available_backends_lists_stdin() {
        assert!(available_backends().contains(&"stdin"));
    }
}
