//! Fire-and-forget notifications.
//!
//! Backed by `libnotify` on X11 / KDE, and by the xdg-desktop-portal
//! notification interface on GNOME / other Wayland compositors
//! (PRD §"Notifications").
//!
//! M1: trait and message enum. The real `libnotify` / portal impls
//! land in M7 alongside clipboard.

use thiserror::Error;

/// Stable identifier for every notification the daemon can emit
/// (PRD §"Notifications" table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationKind {
    TextCopied,
    NoTextFound,
    TesseractMissing,
    DaemonNotRunning,
}

impl NotificationKind {
    pub fn message(self) -> &'static str {
        match self {
            NotificationKind::TextCopied => "✓ Text copied to clipboard",
            NotificationKind::NoTextFound => "No text found in selection.",
            // The Tesseract-missing message is parameterised with the
            // distro-appropriate install command; the sender composes
            // the final string before calling `send`.
            NotificationKind::TesseractMissing => "Tesseract not found. Install with: ",
            NotificationKind::DaemonNotRunning => {
                "Pixelens daemon is not running. Start with: pixelensd"
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum NotifyError {
    #[error("notification backend unavailable: {0}")]
    BackendUnavailable(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Backend-agnostic notifier. All notifications auto-dismiss — no
/// modal dialogs, ever (PRD §"Notifications").
pub trait Notifier: Send + Sync {
    fn send(&self, kind: NotificationKind) -> Result<(), NotifyError>;
}
