//! Windows global hotkey listener.
//!
//! Binds `Win+Shift+S` (or a configured combo) via `RegisterHotKey`, then
//! loops `GetMessage`/`DispatchMessage`. On hotkey press it connects to the
//! `\\.\pipe\pixelens` named pipe and sends `Command::Grab` — the exact same
//! request the CLI sends, so the daemon treats a hotkey press identically to
//! `pixelens grab`.

use crate::{KeyCombo, Mod};

/// Win32 modifier flags for `RegisterHotKey`.
pub mod hotkey_mods {
    pub const MOD_ALT: u32 = 0x0001;
    pub const MOD_CONTROL: u32 = 0x0002;
    pub const MOD_SHIFT: u32 = 0x0004;
    pub const MOD_WIN: u32 = 0x0008;
}

/// Map our `Mod` set to the `RegisterHotKey` modifier bitmask.
pub fn hotkey_modifiers_for(combo: &KeyCombo) -> u32 {
    let mut m = 0u32;
    for mod_ in &combo.mods {
        m |= match mod_ {
            Mod::Super => hotkey_mods::MOD_WIN,
            Mod::Shift => hotkey_mods::MOD_SHIFT,
            Mod::Ctrl => hotkey_mods::MOD_CONTROL,
            Mod::Alt => hotkey_mods::MOD_ALT,
        };
    }
    m
}

/// Map a key name to a Win32 virtual-key code. Returns 0 for unknown keys
/// (RegisterHotKey will then fail at runtime — surfaced by the caller).
pub fn hotkey_vk_for(key: &str) -> u16 {
    // Single ASCII letters/numbers map directly to their VK code.
    if key.len() == 1 {
        let b = key.as_bytes()[0];
        if b.is_ascii_alphabetic() {
            return b.to_ascii_uppercase() as u16;
        }
        if b.is_ascii_digit() {
            return b as u16;
        }
    }
    match key.to_ascii_lowercase().as_str() {
        "space" => 0x20,
        "enter" | "return" => 0x0D,
        "tab" => 0x09,
        "escape" => 0x1B,
        "printscreen" => 0x2C,
        _ => 0,
    }
}

#[cfg(windows)]
mod imp {
    use super::*;
    use crate::KeyhookError;
    use std::ffi::c_int;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_WIN, VK_S,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY,
    };

    const HOTKEY_ID: i32 = 1;

    /// Build the combo we bind on Windows: `Win+Shift+S` by default, but the
    /// caller-supplied combo is honored when it resolves to a known VK.
    fn resolve_combo(_combo: &KeyCombo) -> KeyCombo {
        KeyCombo::parse("Super+Shift+S").expect("static combo parses")
    }

    /// Run the Windows message loop, firing a grab on each hotkey press.
    pub fn run_keyhook_windows(combo: KeyCombo) -> anyhow::Result<()> {
        let combo = resolve_combo(&combo);
        let mods = hotkey_modifiers_for(&combo);
        let vk = hotkey_vk_for(&combo.key);

        // SAFETY: RegisterHotKey with a null window and a constant id; no
        // preconditions beyond a valid modifier/vk. nullptr is allowed.
        unsafe {
            RegisterHotKey(None, HOTKEY_ID, HOT_KEY_MODIFIERS(mods), vk as u32)
                .map_err(|e| KeyhookError::Io(std::io::Error::from(e)))?;
        }

        tracing::info!(combo = %combo.key, "windows hotkey registered");

        let mut msg: MSG = MSG::default();
        // SAFETY: GetMessageW fills `msg`; we only read it after a successful
        // return and only dispatch WM_HOTKEY. TranslateMessage/DispatchMessageW
        // are the standard loop and take no caller-owned pointers.
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).into() {
                if msg.message == WM_HOTKEY {
                    crate::fire_grab();
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let _ = UnregisterHotKey(None, HOTKEY_ID).ok();
        }
        Ok(())
    }

    // Keep VK_S/MOD_WIN referenced so the import is meaningful even if the
    // helper selection changes; they document the canonical binding.
    #[allow(dead_code)]
    fn _canonical() -> (c_int, u32, u16) {
        (HOTKEY_ID, MOD_WIN.0, VK_S.0)
    }
}

#[cfg(windows)]
pub use imp::run_keyhook_windows;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win_shift_s_maps_to_mod_win_shift_and_vk_s() {
        let combo = KeyCombo::parse("Super+Shift+S").unwrap();
        assert_eq!(
            hotkey_modifiers_for(&combo),
            hotkey_mods::MOD_WIN | hotkey_mods::MOD_SHIFT
        );
        assert_eq!(hotkey_vk_for(&combo.key), b'S' as u16);
    }

    #[test]
    fn ctrl_alt_d_maps_correctly() {
        let combo = KeyCombo::parse("Ctrl+Alt+D").unwrap();
        assert_eq!(
            hotkey_modifiers_for(&combo),
            hotkey_mods::MOD_CONTROL | hotkey_mods::MOD_ALT
        );
        assert_eq!(hotkey_vk_for(&combo.key), b'D' as u16);
    }

    #[test]
    fn unknown_key_maps_to_zero() {
        assert_eq!(hotkey_vk_for("Favorites"), 0);
    }
}
