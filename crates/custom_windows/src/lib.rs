//! custom_windows: The crate that defines the UI of the overlay.
//!
//! This crate is responsible for defining the UI of the overlay. It contains the `Window` trait,
//! which every window must implement, and the `regist_windows` function, which assembles and
//! returns a collection of all active UI windows.

use std::sync::OnceLock;
use overlay_types::{HotkeyManager, events::OverlayEvent};
use source_fs::{DummyVpk, P2GameInfo};
use portal2_sdk::Engine;

/// Base font scale factor
pub const BASE_TEXT_SCALE: f32 = 1.25;
/// List of registered window names.
pub static REGISTED_WINDOWS: OnceLock<Vec<&'static str>> = OnceLock::new();

/// --- THE SINGLE SOURCE OF TRUTH ---
/// You can add all your custom mod fields here!
pub struct SharedState {
    pub is_overlay_focused: bool,
    pub allow_inspect_mode: bool,
    pub hotkeys: HotkeyManager,
    pub valve_fs: source_fs::FileSystem<DummyVpk>,

    // Add your custom game-specific fields below:
    // pub something_enabled: bool,
}

impl Default for SharedState {
    fn default() -> Self {
        let game_dir = portal2_sdk::get_engine().engine_server().get_game_dir();
        let valve_fs = source_fs::create_fs_custom::<P2GameInfo, String>(game_dir)
            .expect("Failed to create custom file system");

        Self {
            is_overlay_focused: false,
            allow_inspect_mode: true,
            hotkeys: HotkeyManager::default(),
            valve_fs,
        }
    }
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

/// This function is the designated discovery point for UI components.
/// The core application calls it to populate the `UiManager`'s window list.
pub fn regist(engine: &Engine, shared_state: &mut SharedState) -> Vec<Box<dyn Window + Send>> {
   custom::regist_events(engine, shared_state);
   custom::regist_hotkeys(engine, &mut shared_state.hotkeys);
   let windows = custom::regist_windows(shared_state);
   let _ = REGISTED_WINDOWS.set(windows.iter().map(|w| w.name()).collect());

   windows
}
