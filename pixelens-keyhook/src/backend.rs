//! Backend dispatch: pick the right listener for the detected display server.

use pixelens_capture::DisplayServer;

use crate::{wayland::EvdevListener, x11::X11Listener, KeyCombo, KeyhookError, KeyhookListener};

/// Build a listener for the current display server.
pub fn build(
    display: DisplayServer,
    combo: KeyCombo,
) -> Result<Box<dyn KeyhookListener>, KeyhookError> {
    match display {
        DisplayServer::Wayland => Ok(Box::new(EvdevListener::new(combo)?)),
        DisplayServer::X11 => Ok(Box::new(X11Listener::new(combo)?)),
    }
}
