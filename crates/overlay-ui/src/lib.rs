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

    /// Returns whether the window is open.
    fn is_open(&self) -> bool;

    /// Determines if the window should be rendered in the current frame.
    /// This is checked before calling `draw()`.
    fn is_should_render(&self, _shared_state: &SharedState, _engine: &Engine) -> bool { true }

    /// The drawing logic of the window.
    fn draw(&mut self, ctx: &egui::Context, shared_state: &mut SharedState, engine: &Engine);

    /// Raw input signal processing, optional.
    /// # Returns
    /// * `true` - if the input should be passed to the game.
    /// * `false` - if the input should be "eaten" (blocked).
    fn on_raw_input(&mut self, _umsg: u32, _wparam: u16) -> bool { true }
}


/// Assembles and returns a collection of all active UI windows.
///
/// This function is the designated discovery point for UI components. The core
/// application calls it to populate the `UiManager`'s window list.
pub fn regist_windows() -> Vec<Box<dyn Window + Send>> {
    vec![
        Box::new(OverlayText::default()),
        Box::new(engine_api_demo::EngineApiDemoWindow::default()),
        Box::new(fogui::FogWindow::default()),
    ]
}


// ---------------------- \\
//      YOUR WINDOWS      \\
// ---------------------- \\
// mod debug_win;
mod engine_api_demo;
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
            "IN-Game Custom Overlay",
            egui::FontId::proportional(20.0),
            egui::Color32::ORANGE,
        );
    }
}
