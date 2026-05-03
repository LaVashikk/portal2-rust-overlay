use egui::Context;
use std::sync::OnceLock;
use overlay_types::{HotkeyManager, KeyCode, events::{self, OverlayEvent}};
use portal2_sdk::Engine;

use crate::{SharedState, Window};

pub static REGISTED_WINDOWS: OnceLock<Vec<&'static str>> = OnceLock::new();

/// This function is the designated discovery point for UI components. The core
/// application calls it to populate the `UiManager`'s window list.
pub fn regist(engine: &Engine, shared_state: &mut SharedState) -> Vec<Box<dyn Window + Send>> {
   regist_events(engine, shared_state);
   regist_hotkeys(engine, &mut shared_state.hotkeys);
   let windows = regist_windows();
   let _ = REGISTED_WINDOWS.set(windows.iter().map(|w| w.name()).collect());

   windows
}


pub fn regist_windows() -> Vec<Box<dyn Window + Send>> {
    vec![
        Box::new(OverlayText::default()),
        Box::new(debug_win::DebugWindow::default()),
        Box::new(engine_api_demo::EngineApiDemoWindow::default()),
        Box::new(fogui::FogWindow::default()),
    ]
}


fn regist_events(engine: &Engine, _shared_state: &mut SharedState) {
    // https://developer.valvesoftware.com/wiki/Logic_eventlistener
    engine.game_event_manager().listen("server_spawn", |event| {
        // toater.info(
        //     format!("started. Name: {}, os: {}, hostname: {}", event.get_string("mapname", ""), event.get_string("os", ""), event.get_string("hostname", ""))
        // );
    });
    engine.game_event_manager().listen("server_shutdown", |event| {
        log::warn!("stopped: {}", event.get_string("reason", ""));
    });

    engine.game_event_manager().listen("server_cvar", |event| {
        if event.get_string("cvarname", "") == "developer" {
            let is_enabled = event.get_int("cvarvalue", 0) == 1;
            events::push_event(OverlayEvent::SetWindowState("Debug Window", is_enabled))
        }
    });
}


fn regist_hotkeys(_engine: &Engine, hotkeys_manager: &mut HotkeyManager) {
    hotkeys_manager.bind(KeyCode::F3, OverlayEvent::ToggleOverlay, true);

    hotkeys_manager.bind(KeyCode::Q, OverlayEvent::ToggleWindow("Debug Window"), false);
}


// ---------------------- \\
//      YOUR WINDOWS      \\
// ---------------------- \\
mod debug_win;
mod engine_api_demo;
mod fogui;

#[derive(Default)]
pub struct OverlayText;
impl Window for OverlayText {
    fn name(&self) -> &'static str { "Overlay Text" }
    fn set_open(&mut self, _open: bool) { /* Do nothing */ }
    fn is_open(&self) -> bool { true }

    fn is_should_render(&self, shared_state: &SharedState, _engine: &Engine) -> bool {
        shared_state.is_overlay_focused
    }

    fn draw(&mut self, ctx: &Context, _shared_state: &mut SharedState, _engine: &Engine) {
        let screen_rect = ctx.screen_rect();
        let painter = ctx.debug_painter();

        painter.text(
            egui::pos2(screen_rect.left() + 10.0, screen_rect.bottom() - 10.0),
            egui::Align2::LEFT_BOTTOM,
            "IN-Game Custom Overlay",
            egui::FontId::proportional(20.0),
            egui::Color32::ORANGE,
        );

        let text = "Focus Captured; Press F3 to toggle";
        let font_id = egui::FontId::proportional(24.0);
        let text_color = egui::Color32::WHITE;
        let shadow_color = egui::Color32::BLACK;
        let pos = egui::pos2(screen_rect.center().x, screen_rect.bottom() - 50.0);
        let anchor = egui::Align2::CENTER_BOTTOM;

        // Shadow
        painter.text(
            pos + egui::vec2(2.0, 2.0),
            anchor,
            text,
            font_id.clone(),
            shadow_color,
        );

        // Foreground text
        painter.text(pos, anchor, text, font_id, text_color);
    }
}
