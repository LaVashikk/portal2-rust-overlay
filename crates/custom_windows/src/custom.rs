use overlay_types::{HotkeyManager, KeyCode, events::{self, OverlayEvent}};
use portal2_sdk::Engine;
use crate::{SharedState, Window};

// ---------------------- \\
//      YOUR STUFF        \\
// ---------------------- \\
mod top_panel;
mod demos;
mod tools;

pub(crate) fn regist_windows(shared_state: &mut SharedState) -> Vec<Box<dyn Window + Send>> {
    vec![
        // Custom interface for framework
        Box::new(top_panel::TopPanel::default()),

        // Framework debug windows
        Box::new(demos::SimpleWindow::default()),
        Box::new(demos::EngineApiDemoWindow::default()),
        Box::new(demos::Esp::default()),
        Box::new(demos::DebugOverlayDemo::default()),

        // Useful tools
        Box::new(tools::DebugMenu::default()),
        Box::new(tools::FogWindow::default()),
        Box::new(tools::PostProcessingMenu::default()),
        Box::new(tools::MaterialInspector::new(shared_state)),
        // TODO: sounds emitter window
    ]
}


pub(crate) fn regist_events(engine: &Engine, _shared_state: &mut SharedState) {
    // https://developer.valvesoftware.com/wiki/Logic_eventlistener

    // EXAMPLES:
    // engine.game_event_manager().listen("server_spawn", |event| {
    //     toasts::info(
    //         format!("started. Name: {}, os: {}, hostname: {}", event.get_string("mapname", ""), event.get_string("os", ""), event.get_string("hostname", "")),
    //         4000,
    //     );
    // });

    engine.game_event_manager().listen("server_spawn", |_| {
        events::push_event(OverlayEvent::GameEvent("server_spawn".to_string())); // todo: we lose all event-data
    });

    // engine.game_event_manager().listen("server_cvar", |event| {
    //     if event.get_string("cvarname", "") == "sv_cheats" {
    //         let is_enabled = event.get_int("cvarvalue", 0) == 1;
    //         events::push_event(OverlayEvent::SetWindowState("Simple Window", is_enabled))
    //     }
    // });
}


pub(crate) fn regist_hotkeys(_engine: &Engine, hotkeys_manager: &mut HotkeyManager) {
    hotkeys_manager.bind(KeyCode::F3, OverlayEvent::ToggleOverlay, false);
    hotkeys_manager.bind(KeyCode::F4, OverlayEvent::ToggleWindow("Simple Window"), true);
}
