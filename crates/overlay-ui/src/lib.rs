use egui::Context;
use engine_api::Engine;

/// Shared state accessible to all windows.
#[derive(Debug, Default, Clone)]
pub struct SharedState {
    pub is_overlay_focused: bool,
}

/// Trait that every window must implement.
#[allow(dead_code)]
pub trait Window: std::fmt::Debug { // todo debug
    /// The name of the window, used for the title.
    fn name(&self) -> &'static str;

    /// Shows or hides the window.
    fn toggle(&mut self);
    // fn enable(&mut self); // todo

    /// Returns whether the window is open.
    fn is_open(&self) -> bool;

    /// The drawing logic of the window.
    fn draw(&mut self, ctx: &egui::Context, shared_state: &mut SharedState, engine: &Engine);
}


/// Assembles and returns a collection of all active UI windows.
///
/// This function is the designated discovery point for UI components. The core
/// application calls it to populate the `UiManager`'s window list.
pub fn regist_windows() -> Vec<Box<dyn Window + Send + Sync>> {
    vec![
        Box::new(OverlayText::default()),
        Box::new(debug_win::DebugWindow { is_open: true }),
        Box::new(fogui::FogWindow::default()),
    ]
}


// ---------------------- \\
//      YOUR WINDOWS      \\
// ---------------------- \\
mod debug_win;
mod fogui;

#[derive(Debug, Default)]
pub struct OverlayText;
impl Window for OverlayText {
    fn name(&self) -> &'static str { "Overlay Text" }
    fn toggle(&mut self) {}
    fn is_open(&self) -> bool { true }
    fn draw(&mut self, ctx: &Context, _shared_state: &mut SharedState, _engine: &Engine) {
        let screen_rect = ctx.screen_rect();
        ctx.debug_painter().text(
            egui::pos2(screen_rect.left() + 10.0, screen_rect.bottom() - 10.0),
            egui::Align2::LEFT_BOTTOM,
            "UPDATED CUSTOM OVERLAY!",
            egui::FontId::proportional(20.0),
            egui::Color32::ORANGE,
        );
    }
}
