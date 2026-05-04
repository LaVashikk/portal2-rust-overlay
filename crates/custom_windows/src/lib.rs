//! custom_windows: The crate that defines the UI of the overlay.
//!
//! This crate is responsible for defining the UI of the overlay. It contains the `Window` trait,
//! which every window must implement, and the `regist_windows` function, which assembles and
//! returns a collection of all active UI windows.
use overlay_types::{HotkeyManager, events::OverlayEvent};
use portal2_sdk::Engine;

/// Base font scale factor
pub const BASE_TEXT_SCALE: f32 = 1.25;

/// --- THE SINGLE SOURCE OF TRUTH ---
/// You can add all your custom mod fields here!
#[derive(Default, Clone)]
pub struct SharedState {
    pub is_overlay_focused: bool,
    pub allow_inspect_mode: bool,
    pub hotkeys: HotkeyManager,
    // pub toasts: Toaster,        // TODO: should be GLOBAL!

    // Add your custom game-specific fields below:
    // pub something_enabled: bool,
}

/// Trait that every window must implement.
#[allow(dead_code)]
pub trait Window {
    /// The name of the window, used for the title.
    fn name(&self) -> &'static str;

    /// Shows or hides the window.
    fn set_open(&mut self, _open: bool);

    /// Returns whether the window is open.
    fn is_open(&self) -> bool;

    /// Triggered whenever an event (hotkey, game event, command) is fired.
    fn on_event(&mut self, _event: &OverlayEvent, _shared_state: &mut SharedState) {}

    /// Determines if the window should be rendered in the current frame.
    /// This is checked before calling `draw()`.
    fn is_should_render(&self, _shared_state: &SharedState, _engine: &Engine) -> bool { true }

    /// The drawing logic of the window.
    fn draw(&mut self, ctx: &egui::Context, shared_state: &mut SharedState, engine: &Engine);
}

pub mod custom;
pub use custom::regist;
