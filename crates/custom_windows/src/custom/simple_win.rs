use crate::{SharedState, Window};

#[derive(Debug, Default)]
pub struct SimpleWindow {
    pub is_open: bool,
}

impl Window for SimpleWindow {
    fn name(&self) -> &'static str { "Simple Window" }
    fn set_open(&mut self, open: bool) { self.is_open = open; }
    fn is_open(&self) -> bool { self.is_open }

    fn draw(&mut self, ctx: &egui::Context, _shared_state: &mut SharedState, engine: &portal2_sdk::Engine) {
        egui::Window::new(self.name())
            .collapsible(false)
            .resizable(true)
            .hscroll(true)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.heading("CVar Inspector");
                ui.separator();
                ui.label("I have a keybind set to Q (to open the window). Try pressing it!");

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
