//! Action-bar menu backends for Pixelens.
//!
//! Renders the post-capture "what do you want to do?" choice and encodes the
//! selection as a daemon [`pixelens_ipc::Command`]. Backends: `stdin` (default,
//! headless), `fuzzel`, `wofi` (dmenu-compatible launchers), and a GTK
//! layer-shell `action_bar` (behind the `menu-gtk` feature).
//!
//! The menu owns no side effects: it produces a [`MenuChoice`] and the caller
//! turns that into an [`pixelens_ipc::IpcRequest`] via
//! [`MenuChoice::to_command`]/[`MenuChoice::to_request`].

pub mod action_bar;
pub mod backend;
pub mod fuzzel;
pub mod stdin;
pub mod types;
pub mod wofi;

pub use backend::{available_backends, create_backend, detect_backend};
pub use types::{MenuBackend, MenuChoice, MenuError};

/// Serializes the PATH-mutating shim tests across modules. Tests that prepend a
/// fake launcher to `PATH` must hold this guard for their whole body, otherwise
/// concurrent tests corrupt each other's `PATH` (and the spawned binary).
#[cfg(test)]
pub(crate) fn path_test_lock() -> &'static std::sync::Mutex<()> {
    use std::sync::{Mutex, OnceLock};
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
