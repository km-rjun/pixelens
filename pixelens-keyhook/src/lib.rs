//! `pixelens-keyhook` — global hotkey listener for Pixelens.
//!
//! The listener is deliberately dumb: on a configured key combo it connects
//! to the running `pixelensd` daemon over the **existing Unix socket** and
//! sends `Command::Grab` — exactly what the CLI does. All capture/OCR state
//! stays in the daemon. This keeps a single code path for "trigger a grab"
//! and means the hotkey listener cannot block or slow the capture loop.
//!
//! Backend selection mirrors the daemon's display-server detection:
//! Wayland → evdev event-device reader; X11 → x11rb root-window listener.

pub mod backend;
pub mod wayland;
pub mod x11;

use std::collections::HashSet;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyhookError {
    #[error("no key combo configured")]
    NoCombo,

    #[error("invalid modifier '{0}' (want Super, Shift, Ctrl, or Alt)")]
    BadModifier(String),

    #[error("could not open any input device: are you in the 'input' group? ({0})")]
    EvdevUnavailable(String),

    #[error("x11 connection failed: {0}")]
    X11Connect(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// A parsed hotkey combo, e.g. `Super+Shift+S`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCombo {
    pub mods: HashSet<Mod>,
    /// Linux key name as understood by the backend (evdev `KEY_*` or
    /// x11rb `keysym`). We keep it as a string and resolve per-backend.
    pub key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mod {
    Super,
    Shift,
    Ctrl,
    Alt,
}

impl KeyCombo {
    /// Parse `"Super+Shift+S"` → `KeyCombo { mods: {Super, Shift}, key: "S" }`.
    /// Modifiers are case-insensitive; the final token is the trigger key.
    pub fn parse(s: &str) -> Result<Self, KeyhookError> {
        let parts: Vec<&str> = s
            .split('+')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
        if parts.len() < 2 {
            return Err(KeyhookError::NoCombo);
        }
        let key = parts.last().unwrap().to_string();
        let mut mods = HashSet::new();
        for m in &parts[..parts.len() - 1] {
            let m = match m.to_ascii_lowercase().as_str() {
                "super" | "meta" | "win" => Mod::Super,
                "shift" => Mod::Shift,
                "ctrl" | "control" => Mod::Ctrl,
                "alt" => Mod::Alt,
                other => return Err(KeyhookError::BadModifier(other.to_string())),
            };
            mods.insert(m);
        }
        Ok(KeyCombo { mods, key })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_super_shift_s() {
        let c = KeyCombo::parse("Super+Shift+S").unwrap();
        assert!(c.mods.contains(&Mod::Super));
        assert!(c.mods.contains(&Mod::Shift));
        assert_eq!(c.key, "S");
    }

    #[test]
    fn rejects_bare_key() {
        assert!(KeyCombo::parse("S").is_err());
    }

    #[test]
    fn rejects_unknown_mod() {
        assert!(KeyCombo::parse("Foo+S").is_err());
    }
}

/// A running hotkey listener. Implemented per display server.
pub trait KeyhookListener {
    /// Block until stopped (Ctrl-C / process kill). On combo press, fires
    /// a grab via [`fire_grab`].
    fn run(self: Box<Self>) -> anyhow::Result<()>;
}

/// Connect to the daemon socket and send `Command::Grab`. This is the exact
/// same request the CLI sends, so the daemon treats a hotkey press
/// identically to `pixelens grab`.
///
/// Failures are logged, not propagated — a dead daemon or a failed grab must
/// never crash the listener.
pub fn fire_grab() {
    use pixelens_ipc::{write_frame, Command, IpcRequest};
    use tokio::net::UnixStream;
    use uuid::Uuid;

    tracing::debug!("hotkey pressed -> sending Grab");
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            tracing::warn!(error = %e, "could not build runtime for grab");
            return;
        }
    };
    rt.block_on(async {
        let path = socket_path();
        let mut stream = match UnixStream::connect(&path).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, socket = %path.display(), "daemon not reachable");
                return;
            }
        };
        let req = IpcRequest {
            request_id: Uuid::new_v4().to_string(),
            command: Command::Grab,
            payload: serde_json::json!({}),
        };
        if let Err(e) = write_frame(&req, &mut stream).await {
            tracing::warn!(error = %e, "failed to send Grab frame");
        }
    });
}

/// Resolve the daemon socket path. Mirrors `pixelens-cli/src/main.rs`.
fn socket_path() -> std::path::PathBuf {
    if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        if !dir.is_empty() {
            return std::path::PathBuf::from(dir).join("pixelens.sock");
        }
    }
    #[cfg(unix)]
    {
        extern "C" {
            fn getuid() -> u32;
        }
        // SAFETY: getuid is async-signal-safe and has no preconditions.
        let uid = unsafe { getuid() };
        std::path::PathBuf::from(format!("/tmp/pixelens-{uid}.sock"))
    }
    #[cfg(not(unix))]
    {
        std::path::PathBuf::from("(no socket on non-unix)")
    }
}
