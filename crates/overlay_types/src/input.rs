use std::collections::HashMap;
use crate::events::{OverlayEvent, push_event};

/// High-level representation of keyboard keys, abstracting away WinAPI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Space, Enter, Escape, Shift, Ctrl, Alt, Insert, Delete, Home, End, PageUp, PageDown,
    Unknown,
}

impl KeyCode {
    /// Translates raw WinAPI virtual key codes (WPARAM) into framework's KeyCode.
    pub fn from_winapi(wparam: u16) -> Self {
        match wparam {
            0x41..=0x5A => unsafe { std::mem::transmute((wparam - 0x41) as u8) },
            0x70 => KeyCode::F1,  0x71 => KeyCode::F2,  0x72 => KeyCode::F3,  0x73 => KeyCode::F4,
            0x74 => KeyCode::F5,  0x75 => KeyCode::F6,  0x76 => KeyCode::F7,  0x77 => KeyCode::F8,
            0x78 => KeyCode::F9,  0x79 => KeyCode::F10, 0x7A => KeyCode::F11, 0x7B => KeyCode::F12,
            0x20 => KeyCode::Space, 0x0D => KeyCode::Enter, 0x1B => KeyCode::Escape,
            0x10 => KeyCode::Shift, 0x11 => KeyCode::Ctrl,  0x12 => KeyCode::Alt,
            0x2D => KeyCode::Insert, 0x2E => KeyCode::Delete,
            0x24 => KeyCode::Home,  0x23 => KeyCode::End,
            0x21 => KeyCode::PageUp, 0x22 => KeyCode::PageDown,
            _ => KeyCode::Unknown,
        }
    }
}

/// Manages global hotkeys and routes them to framework events.
#[derive(Default, Clone)]
pub struct HotkeyManager {
    pub binds: HashMap<KeyCode, (OverlayEvent, bool)>,
}

impl HotkeyManager {
    /// Bind a specific KeyCode to trigger an OverlayEvent.
    pub fn bind(&mut self, key: KeyCode, event: OverlayEvent, pass_to_game: bool) {
        self.binds.insert(key, (event, pass_to_game));
    }

    /// Remove a specific KeyCode binding.
    pub fn remove(&mut self, key: KeyCode) {
        self.binds.remove(&key);
    }

    /// Fire a bound event for a specific KeyCode.
    pub fn fire_bind(&self, key: &KeyCode) {
        if let Some((event, _)) = self.binds.get(key) {
            push_event(event.clone());
        }
    }
}
