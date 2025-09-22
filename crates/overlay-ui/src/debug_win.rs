use crate::{SharedState, Window};

#[derive(Debug)]
pub struct DebugWindow {
    pub is_open: bool,
}

impl Window for DebugWindow {
    fn name(&self) -> &'static str { "Debug Window" }
    fn toggle(&mut self) { self.is_open = !self.is_open; }
    fn is_open(&self) -> bool { self.is_open }

    fn draw(&mut self, ctx: &egui::Context, shared_state: &mut SharedState, engine: &engine_api::Engine) {
        if !shared_state.is_overlay_focused {
            return
        }

        egui::Window::new(self.name())
            .collapsible(false)
            .resizable(true)
            .hscroll(true)
            .vscroll(true)
            .default_height(38.)
            .show(ctx, |ui| {
                ui.heading("CVar Inspector");
                ui.separator();

                let cvar_system = engine.cvar_system();
                match cvar_system.find_var("sv_cheats") {
                    Some(sv_cheats_cvar) => {
                        let value = sv_cheats_cvar.get_int();

                        ui.label(format!("sv_cheats value: {}", value));

                        if ui.button("Toggle sv_cheats").clicked() {
                           let new_value = if value == 0 { 1 } else { 0 };
                           engine.client()
                                .execute_client_cmd_unrestricted(&format!("sv_cheats {}", new_value));
                        }
                    }
                    None => {
                        // Impossible case, this should not happen!
                        ui.colored_label(egui::Color32::RED, "sv_cheats: <not found>");
                    }
                }
            });
    }
}
