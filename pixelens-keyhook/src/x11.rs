//! X11 hotkey backend.
//!
//! Uses `x11rb` to open a connection, grab the configured key combination on
//! the root window, and fire `Grab` on match. Pure-Rust, no shell-out.

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt, ModMask, Window};
use x11rb::protocol::Event;

use crate::{KeyCombo, KeyhookError, KeyhookListener, Mod};

pub struct X11Listener {
    combo: KeyCombo,
    mod_mask: u16,
    /// X11 keysym for the trigger key.
    keysym: u32,
}

impl X11Listener {
    pub fn new(combo: KeyCombo) -> Result<Self, KeyhookError> {
        let mut mod_mask: u16 = 0;
        for m in &combo.mods {
            mod_mask |= match m {
                Mod::Super => ModMask::M4,
                Mod::Shift => ModMask::SHIFT,
                Mod::Ctrl => ModMask::CONTROL,
                Mod::Alt => ModMask::M1,
            };
        }
        let keysym = key_name_to_keysym(&combo.key)
            .ok_or_else(|| KeyhookError::BadModifier(combo.key.clone()))?;
        Ok(Self {
            combo,
            mod_mask,
            keysym,
        })
    }

    /// Build a reverse keysym→keycode map from the server's keyboard mapping
    /// and return the keycode for `self.keysym`.
    fn resolve_keycode<C: Connection>(&self, conn: &C) -> Result<u8, KeyhookError> {
        let setup = conn.setup();
        let min = setup.min_keycode;
        let max = setup.max_keycode;
        let count = max - min + 1;
        let reply = conn
            .get_keyboard_mapping(min, count)
            .map_err(|e| KeyhookError::X11Connect(e.to_string()))?
            .reply()
            .map_err(|e| KeyhookError::X11Connect(e.to_string()))?;

        let per = reply.keysyms_per_keycode as usize;
        let syms = &reply.keysyms;
        for (i, chunk) in syms.chunks(per).enumerate() {
            for &ks in chunk.iter().take(per) {
                if ks == self.keysym {
                    return Ok(min + i as u8);
                }
            }
        }
        Ok(0)
    }
}

impl KeyhookListener for X11Listener {
    fn run(self: Box<Self>) -> anyhow::Result<()> {
        let (conn, screen_num) =
            x11rb::connect(None).map_err(|e| KeyhookError::X11Connect(e.to_string()))?;
        let root: Window = conn.setup().roots[screen_num].root;

        // Resolve the keycode for the keysym by walking the keyboard mapping.
        let keycode = self.resolve_keycode(&conn)?;
        if keycode == 0 {
            return Err(KeyhookError::X11Connect(format!(
                "no keycode for keysym {:#x}",
                self.keysym
            ))
            .into());
        }

        // Grab the key on the root window (any window, with the modifiers).
        conn.grab_key(
            false,
            root,
            self.mod_mask.into(),
            keycode,
            xproto::GrabMode::ASYNC,
            xproto::GrabMode::ASYNC,
        )?;
        conn.flush()?;

        tracing::info!(combo = %format_combo(&self.combo), "x11 hotkey listener active");

        loop {
            let event = conn.wait_for_event()?;
            if let Event::KeyPress(kp) = event {
                let state: u16 = kp.state.bits();
                if kp.detail == keycode && (state & 0xFF) == self.mod_mask {
                    crate::fire_grab();
                }
            }
        }
    }
}

/// Minimal ASCII letter/digit → keysym (lowercase letter keysyms start at 0x61).
fn key_name_to_keysym(name: &str) -> Option<u32> {
    let upper = name.to_ascii_uppercase();
    let c = upper.chars().next()?;
    if c.is_ascii_alphabetic() || c.is_ascii_digit() {
        // X11 uses ASCII codepoint for letters/digits.
        return Some(c as u32);
    }
    None
}

fn format_combo(c: &KeyCombo) -> String {
    let mut parts: Vec<String> = c.mods.iter().map(|m| format!("{m:?}")).collect();
    parts.push(c.key.clone());
    parts.join("+")
}
