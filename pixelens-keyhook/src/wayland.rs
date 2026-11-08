//! Wayland hotkey backend.
//!
//! Wayland has no broadly-shipped global-hotkey protocol, so we read raw
//! key events from `/dev/input/event*` devices via the `evdev` crate.
//! We deliberately do **NOT** grab the device (`EVIOCGRAB`) — we only
//! *observe*, so the rest of the desktop keeps working. The user must be
//! in the `input` group for these devices to be readable.
//!
//! We open every event device that exposes keys, track modifier state
//! across all of them, and fire when the combo's modifiers are down and
//! the trigger key goes down.

use std::collections::HashSet;

use evdev::{Device, EventType, InputEventKind, Key};

use crate::{KeyCombo, KeyhookError, KeyhookListener, Mod};

pub struct EvdevListener {
    combo: KeyCombo,
    /// evdev key codes for the modifiers we care about (resolved once).
    mod_codes: HashSet<Key>,
    /// evdev key code for the trigger key (best-effort; may be None if
    /// the string doesn't map, in which case we never fire — logged).
    trigger: Option<Key>,
}

impl EvdevListener {
    pub fn new(combo: KeyCombo) -> Result<Self, KeyhookError> {
        let mod_codes: HashSet<Key> = combo.mods.iter().filter_map(|m| mod_to_evdev(*m)).collect();
        let trigger = key_name_to_evdev(&combo.key);
        if trigger.is_none() {
            tracing::warn!(
                key = %combo.key,
                "trigger key not recognised by evdev mapping; hotkey will never fire"
            );
        }
        Ok(Self {
            combo,
            mod_codes,
            trigger,
        })
    }

    fn open_devices() -> Result<Vec<Device>, KeyhookError> {
        let mut devices = Vec::new();
        // enumerate via /dev/input/event* (no libevdev enumerate dep needed)
        let entries = std::fs::read_dir("/dev/input")
            .map_err(|e| KeyhookError::EvdevUnavailable(e.to_string()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with("event") {
                continue;
            }
            match Device::open(&path) {
                Ok(dev) => {
                    // Only keep devices that have keys (keyboards / mice).
                    if dev.supported_keys().map(|k| k.iter().count()).unwrap_or(0) > 0 {
                        devices.push(dev);
                    }
                }
                Err(e) => {
                    tracing::debug!(path = %path.display(), error = %e, "could not open device");
                }
            }
        }
        if devices.is_empty() {
            return Err(KeyhookError::EvdevUnavailable(
                "no readable event devices in /dev/input".into(),
            ));
        }
        Ok(devices)
    }
}

impl KeyhookListener for EvdevListener {
    fn run(self: Box<Self>) -> anyhow::Result<()> {
        let mut devices = Self::open_devices()?;
        tracing::info!(
            combo = %format_combo(&self.combo),
            devices = devices.len(),
            "evdev hotkey listener active"
        );

        let mut down_mods: HashSet<Key> = HashSet::new();
        loop {
            // Poll each device; evdev::Device::fetch_events blocks per device,
            // so we use a non-blocking aggregate loop with a small sleep to
            // avoid busy-spinning while staying simple (no epoll crate).
            for dev in &mut devices {
                match dev.fetch_events() {
                    Ok(events) => {
                        for ev in events {
                            if ev.event_type() != EventType::KEY {
                                continue;
                            }
                            if let InputEventKind::Key(key) = ev.kind() {
                                let pressed = ev.value() == 1;
                                if self.mod_codes.contains(&key) {
                                    if pressed {
                                        down_mods.insert(key);
                                    } else {
                                        down_mods.remove(&key);
                                    }
                                } else if Some(key) == self.trigger
                                    && pressed
                                    && self.mod_codes.iter().all(|m| down_mods.contains(m))
                                {
                                    crate::fire_grab();
                                }
                            }
                        }
                    }
                    Err(_) => { /* would-block or transient; continue */ }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(8));
        }
    }
}

fn mod_to_evdev(m: Mod) -> Option<Key> {
    Some(match m {
        Mod::Super => Key::KEY_LEFTMETA,
        Mod::Shift => Key::KEY_LEFTSHIFT,
        Mod::Ctrl => Key::KEY_LEFTCTRL,
        Mod::Alt => Key::KEY_LEFTALT,
    })
}

/// Minimal map of common letter/number keys to evdev codes. Extend as needed.
fn key_name_to_evdev(name: &str) -> Option<Key> {
    let upper = name.to_ascii_uppercase();
    let c = upper.chars().next()?;
    if c.is_ascii_alphabetic() {
        // KEY_A = 30, KEY_Z = 55; 'A' -> 30
        let code = 30 + (c as u8 - b'A') as u16;
        return Some(Key::new(code));
    }
    if c.is_ascii_digit() {
        // KEY_0 = 11 ... KEY_9 = 20
        let code = 11 + (c as u8 - b'0') as u16;
        return Some(Key::new(code));
    }
    None
}

fn format_combo(c: &KeyCombo) -> String {
    let mut parts: Vec<String> = c.mods.iter().map(|m| format!("{m:?}")).collect();
    parts.push(c.key.clone());
    parts.join("+")
}
