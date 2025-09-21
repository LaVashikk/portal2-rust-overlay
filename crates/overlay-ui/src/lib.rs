use egui::Context;

/// Shared state accessible to all windows.
#[derive(Debug, Default, Clone)]
pub struct SharedState {
    pub is_overlay_focused: bool, // todo: namening?
    pub open_counter: u64,
}

/// Trait that every window must implement.
#[allow(dead_code)]
pub trait Window: std::fmt::Debug { // todo debug
    /// The name of the window, used for the title.
    fn name(&self) -> &'static str;

    /// Shows or hides the window.
    fn toggle(&mut self);
    /// todo
    // fn enable(&mut self);

    /// Returns whether the window is open.
    fn is_open(&self) -> bool;

    /// The drawing logic of the window.
    fn draw(&mut self, ctx: &egui::Context, shared_state: &mut SharedState);
}


// ---------------------- \\
//      YOUR WINDOWS      \\
// ---------------------- \\

#[derive(Debug, Default)]
pub struct OverlayText;
impl Window for OverlayText {
    fn name(&self) -> &'static str { "Overlay Text" }
    fn toggle(&mut self) {}
    fn is_open(&self) -> bool { true }
    fn draw(&mut self, ctx: &Context, _shared_state: &mut SharedState) {
        let screen_rect = ctx.screen_rect();
        ctx.debug_painter().text(
            egui::pos2(screen_rect.left() + 10.0, screen_rect.bottom() - 10.0),
            egui::Align2::LEFT_BOTTOM,
            "IN-GAME CUSTOM OVERLAY!",
            egui::FontId::proportional(20.0),
            egui::Color32::ORANGE,
        );
    }
}


// #[derive(Debug)]
// pub struct DebugWindow {
//     is_open: bool,
// }

// impl Window for DebugWindow {
//     fn name(&self) -> &'static str { "Debug Window" }
//     fn toggle(&mut self) { self.is_open = !self.is_open; }
//     fn is_open(&self) -> bool { self.is_open }

//     fn draw(&mut self, ctx: &Context, shared_state: &mut SharedState) {
//         if !shared_state.is_overlay_focused {
//             return
//         }

//         egui::Window::new(self.name())
//             .open(&mut self.is_open)
//             .resizable(true)
//             .show(ctx, |ui| {
//                 ui.heading("CVar Inspector");
//                 ui.separator();

//                 let cvar_system = engine_api::get().cvar_system();
//                 match cvar_system.find_var("sv_cheats") {
//                     Some(sv_cheats_cvar) => {
//                         let value = sv_cheats_cvar.get_int();

//                         ui.label(format!("sv_cheats value: {}", value));

//                         if ui.button("Toggle sv_cheats").clicked() {
//                            let new_value = if value == 0 { 1 } else { 0 };
//                            engine_api::get()
//                                 .client()
//                                 .execute_client_cmd_unrestricted(&format!("sv_cheats {}", new_value));
//                         }
//                     }
//                     None => {
//                         // Impossible case, this should not happen!
//                         ui.colored_label(egui::Color32::RED, "sv_cheats: <not found>");
//                     }
//                 }
//             });
//     }
// }

// mod fogui;
