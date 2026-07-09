use std::sync::mpsc;
use std::sync::OnceLock;

use crate::KeyCode;

/// The universal event enum for the overlay framework.
#[derive(Debug, Clone)]
pub enum OverlayEvent {
    /// Toggles the overlay's visibility.
    ToggleOverlay,
    /// Set overlay system state.
    SetOverlayFocus(bool),

    /// Toggle a window's visibility by its name.
    ToggleWindow(&'static str),
    /// Explicitly set a window's visibility.
    SetWindowState(&'static str, bool),
    /// Closes all active windows.
    CloseAllWindows,
    /// Opens all closed windows.
    OpenAllWindows,

    /// Sends a command to the game engine.
    EngineCommand(String),

    /// Game events triggered by the Source Engine.
    GameEvent(String),

    /// Presses a key.
    PressKey(KeyCode), // todo: only for UI for now

    /// Custom commands triggered by hotkeys or UI.
    Command(String),
}

/// Global event sender. Allows cross-thread event pushing
pub static EVENT_SENDER: OnceLock<mpsc::Sender<OverlayEvent>> = OnceLock::new();

/// Push a new event to the global event bus
pub fn push_event(event: OverlayEvent) {
    if let Some(sender) = EVENT_SENDER.get() {
        let _ = sender.send(event);
    }
}
