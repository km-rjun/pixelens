//! Menu domain types: the user's action choice, the backend trait, and the
//! mapping from a choice to a daemon [`Command`] + payload.
//!
//! The menu crate is deliberately transport-agnostic: it knows how to *render*
//! a choice (via a [`MenuBackend`]) and how to *encode* that choice as the
//! pieces the daemon consumes ([`MenuChoice::to_command`]), but it never
//! performs the action itself. The daemon owns all side effects (clipboard,
//! network, search), so the menu stays trivially testable and free of heavy
//! backend deps.
//!
//! Wire shape note: in `pixelens_ipc`, [`Command`] is a flat enum and the
//! request's payload rides alongside it on [`IpcRequest::payload`] (a free-form
//! `serde_json::Value`). So [`MenuChoice::to_command`] returns the command and
//! its already-serialized payload as a pair; callers assemble the full
//! [`IpcRequest`] (supplying a `request_id`). [`MenuChoice::to_request`] is the
//! convenience wrapper that does exactly that.

use pixelens_ipc::{AiPayload, Command, CopyPayload, IpcRequest, SearchPayload, TranslatePayload};
use thiserror::Error;

/// A user's selection from the action menu, surfaced after an OCR capture.
///
/// `Cancel` is the distinguished "do nothing" choice and maps to
/// [`Command::Cancel`] so the daemon can short-circuit the in-flight session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuChoice {
    /// Copy the OCR text to the system clipboard.
    Copy,
    /// Web-search the OCR text.
    Search,
    /// Send the OCR text to the configured AI model.
    Ai,
    /// Translate the OCR text.
    Translate,
    /// Abort — run no action.
    Cancel,
}

impl MenuChoice {
    /// Parse a single-keystroke choice (case-insensitive). Returns `None` for
    /// anything that is not a known key, so callers can reject or re-prompt.
    ///
    /// Keys: `c` copy, `s` search, `a` ask-AI, `t` translate, and any of
    /// `escape`/`esc`/`q`/empty as cancel.
    pub fn from_key(key: &str) -> Option<Self> {
        match key.trim().to_lowercase().as_str() {
            "c" => Some(MenuChoice::Copy),
            "s" => Some(MenuChoice::Search),
            "a" => Some(MenuChoice::Ai),
            "t" => Some(MenuChoice::Translate),
            "escape" | "esc" | "q" | "" => Some(MenuChoice::Cancel),
            _ => None,
        }
    }

    /// The short key bound to this choice (used by backends that render a
    /// key hint, e.g. `[C] Copy`).
    pub fn key(&self) -> &'static str {
        match self {
            MenuChoice::Copy => "c",
            MenuChoice::Search => "s",
            MenuChoice::Ai => "a",
            MenuChoice::Translate => "t",
            MenuChoice::Cancel => "esc",
        }
    }

    /// Encode this choice as a daemon [`Command`] plus its `serde_json::Value`
    /// payload, embedding the OCR text the user just captured.
    ///
    /// The payload is returned separately because in `pixelens_ipc` the
    /// command is a flat enum and the payload rides on [`IpcRequest::payload`].
    /// Callers assemble the full request (and supply a `request_id`) via
    /// [`MenuChoice::to_request`].
    ///
    /// `Cancel` carries an explicit `serde_json::Value::Null` payload (it never
    /// needs one), keeping the return shape uniform. `Translate` defaults to
    /// English, mirroring the alternate implementation's `Translate("English")`
    /// baseline; the daemon may learn a per-user target language later.
    pub fn to_command(self, ocr_text: &str) -> (Command, serde_json::Value) {
        let payload = match self {
            MenuChoice::Copy => serde_json::to_value(CopyPayload {
                text: ocr_text.to_string(),
            })
            .expect("CopyPayload is always serializable"),
            MenuChoice::Search => serde_json::to_value(SearchPayload {
                text: ocr_text.to_string(),
            })
            .expect("SearchPayload is always serializable"),
            MenuChoice::Ai => serde_json::to_value(AiPayload {
                prompt: ocr_text.to_string(),
                image_path: None,
            })
            .expect("AiPayload is always serializable"),
            MenuChoice::Translate => serde_json::to_value(TranslatePayload {
                text: ocr_text.to_string(),
                target_lang: "English".to_string(),
            })
            .expect("TranslatePayload is always serializable"),
            MenuChoice::Cancel => serde_json::Value::Null,
        };
        (self.command(), payload)
    }

    /// Convenience: build a full [`IpcRequest`] from this choice.
    ///
    /// `request_id` is supplied by the caller (the daemon/CLI owns id
    /// generation); pass a fresh UUID or any unique string.
    pub fn to_request(self, ocr_text: &str, request_id: impl Into<String>) -> IpcRequest {
        let (command, payload) = self.to_command(ocr_text);
        IpcRequest {
            request_id: request_id.into(),
            command,
            payload,
        }
    }

    /// The bare [`Command`] variant for this choice, without payload.
    fn command(self) -> Command {
        match self {
            MenuChoice::Copy => Command::Copy,
            MenuChoice::Search => Command::Search,
            MenuChoice::Ai => Command::Ai,
            MenuChoice::Translate => Command::Translate,
            MenuChoice::Cancel => Command::Cancel,
        }
    }
}

/// Errors raised by menu backends and the factory.
///
/// These are local to the menu crate; we do not import the daemon's error type
/// so the crate stays decoupled from the rest of the IPC transport.
#[derive(Debug, Error)]
pub enum MenuError {
    /// A backend could not be launched or returned an unexpected result.
    #[error("menu backend error: {0}")]
    Backend(String),
    /// The channel from a graphical backend closed before a choice arrived.
    #[error("menu channel closed: {0}")]
    ChannelClosed(String),
    /// Reading from stdin failed.
    #[error("menu io error: {0}")]
    Io(#[from] std::io::Error),
    /// Any other backend-specific failure.
    #[error("{0}")]
    Other(String),
}

/// A renderer for the action menu. Each backend returns the user's
/// [`MenuChoice`] given the OCR text it refers to.
///
/// Implementations must be cheap to construct (the factory hands out
/// `Box<dyn MenuBackend>`); all expensive setup (spawning a GUI event loop)
/// happens inside [`show_menu`](MenuBackend::show_menu).
pub trait MenuBackend {
    /// Present the menu and block until the user picks an action.
    fn show_menu(&self, ocr_text: &str) -> Result<MenuChoice, MenuError>;
    /// Stable backend identifier (e.g. `"stdin"`, `"fuzzel"`).
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_key_maps_all_choices() {
        assert_eq!(MenuChoice::from_key("c"), Some(MenuChoice::Copy));
        assert_eq!(MenuChoice::from_key("C"), Some(MenuChoice::Copy));
        assert_eq!(MenuChoice::from_key("s"), Some(MenuChoice::Search));
        assert_eq!(MenuChoice::from_key("S"), Some(MenuChoice::Search));
        assert_eq!(MenuChoice::from_key("a"), Some(MenuChoice::Ai));
        assert_eq!(MenuChoice::from_key("A"), Some(MenuChoice::Ai));
        assert_eq!(MenuChoice::from_key("t"), Some(MenuChoice::Translate));
        assert_eq!(MenuChoice::from_key("T"), Some(MenuChoice::Translate));
        assert_eq!(MenuChoice::from_key("escape"), Some(MenuChoice::Cancel));
        assert_eq!(MenuChoice::from_key("esc"), Some(MenuChoice::Cancel));
        assert_eq!(MenuChoice::from_key("q"), Some(MenuChoice::Cancel));
        assert_eq!(MenuChoice::from_key(""), Some(MenuChoice::Cancel));
        assert_eq!(MenuChoice::from_key("x"), None);
        assert_eq!(MenuChoice::from_key("5"), None);
    }

    #[test]
    fn copy_never_maps_to_search() {
        let copy = MenuChoice::from_key("c");
        let search = MenuChoice::from_key("s");
        assert_eq!(copy, Some(MenuChoice::Copy));
        assert_eq!(search, Some(MenuChoice::Search));
        assert_ne!(copy, search);
    }

    #[test]
    fn to_command_returns_correct_variant_per_choice() {
        let text = "hello world";
        assert_eq!(MenuChoice::Copy.to_command(text).0, Command::Copy);
        assert_eq!(MenuChoice::Search.to_command(text).0, Command::Search);
        assert_eq!(MenuChoice::Ai.to_command(text).0, Command::Ai);
        assert_eq!(MenuChoice::Translate.to_command(text).0, Command::Translate);
        assert_eq!(MenuChoice::Cancel.to_command(text).0, Command::Cancel);
    }

    #[test]
    fn to_command_payloads_round_trip() {
        let text = "the quick brown fox";
        let (_, copy_p) = MenuChoice::Copy.to_command(text);
        let payload: CopyPayload = serde_json::from_value(copy_p).unwrap();
        assert_eq!(payload.text, text);

        let (_, search_p) = MenuChoice::Search.to_command(text);
        let sp: SearchPayload = serde_json::from_value(search_p).unwrap();
        assert_eq!(sp.text, text);

        let (_, ai_p) = MenuChoice::Ai.to_command(text);
        let ap: AiPayload = serde_json::from_value(ai_p).unwrap();
        assert_eq!(ap.prompt, text);
        assert_eq!(ap.image_path, None);

        let (_, tr_p) = MenuChoice::Translate.to_command(text);
        let tp: TranslatePayload = serde_json::from_value(tr_p).unwrap();
        assert_eq!(tp.text, text);
        assert_eq!(tp.target_lang, "English");

        let (_, cancel_p) = MenuChoice::Cancel.to_command(text);
        assert_eq!(cancel_p, serde_json::Value::Null);
    }

    #[test]
    fn to_request_assembles_full_ipc_request() {
        let req = MenuChoice::Search.to_request("find me", "req-123");
        assert_eq!(req.request_id, "req-123");
        assert_eq!(req.command, Command::Search);
        let sp: SearchPayload = serde_json::from_value(req.payload).unwrap();
        assert_eq!(sp.text, "find me");
    }
}
