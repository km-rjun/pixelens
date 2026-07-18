//! Fire-and-forget notifications.
//!
//! Backed by `libnotify` on X11 / KDE, and by the xdg-desktop-portal
//! notification interface on GNOME / other Wayland compositors
//! (PRD §"Notifications").
//!
//! M7: a dependency-light `notify-send` backend (`NotifySend`) is
//! provided — it shells out to the `notify-send` binary (part of
//! `libnotify`), which is the lowest-friction way to pop a desktop
//! notification on both Wayland and X11 without pulling in a native
//! binding. A native `libnotify` / portal impl can replace it later
//! without changing call sites (the [`Notifier`] trait is the API).

use std::process::Command;

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

/// Dependency-light notification backend that shells out to
/// `notify-send` (from the `libnotify` package).
///
/// This is the M7 implementation of [`Notifier`]. It is intentionally
/// non-fatal: if `notify-send` is not installed, [`Notifier::send`]
/// returns [`NotifyError::BackendUnavailable`] and the caller is
/// expected to log-and-continue rather than abort the grab.
pub struct NotifySend;

impl NotifySend {
    /// Construct a new `notify-send` backend.
    pub fn new() -> Self {
        Self
    }

    /// Returns `true` if the `notify-send` binary is resolvable on
    /// `$PATH`. Callers may use this to decide whether to even
    /// attempt a notification, though [`Notifier::send`] already
    /// degrades gracefully when it is missing.
    pub fn available() -> bool {
        Command::new("notify-send")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for NotifySend {
    fn default() -> Self {
        Self::new()
    }
}

impl Notifier for NotifySend {
    fn send(&self, kind: NotificationKind) -> Result<(), NotifyError> {
        let message = kind.message();
        let summary = match kind {
            NotificationKind::TextCopied => "Pixelens",
            NotificationKind::NoTextFound => "Pixelens",
            NotificationKind::TesseractMissing => "Pixelens",
            NotificationKind::DaemonNotRunning => "Pixelens",
        };

        let status = Command::new("notify-send")
            .arg("--app-name")
            .arg("Pixelens")
            .arg(summary)
            .arg(message)
            .status()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    NotifyError::BackendUnavailable("notify-send not found on PATH".to_string())
                } else {
                    NotifyError::Io(e)
                }
            })?;

        if status.success() {
            Ok(())
        } else {
            Err(NotifyError::BackendUnavailable(format!(
                "notify-send exited with status {status}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_strings_are_stable() {
        assert_eq!(
            NotificationKind::TextCopied.message(),
            "✓ Text copied to clipboard"
        );
        assert_eq!(
            NotificationKind::NoTextFound.message(),
            "No text found in selection."
        );
    }
}
