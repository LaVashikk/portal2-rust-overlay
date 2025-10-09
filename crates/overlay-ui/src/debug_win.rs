use crate::{SharedState, Window};

#[derive(Debug, Default)]
pub struct DebugWindow {
    pub is_closed: bool,
    force_draw: bool,
}

impl Window for DebugWindow {
    fn name(&self) -> &'static str { "Debug Window" }
    fn toggle(&mut self) { self.is_closed = !self.is_closed; }
    fn is_open(&self) -> bool { !self.is_closed }
    fn is_should_render(&self, shared_state: &SharedState, _engine: &engine_api::Engine) -> bool {
        shared_state.is_overlay_focused || self.force_draw
    }

    fn on_raw_input(&mut self, umsg: u32, wparam: u16) -> bool {
        // Handle keyup messages.
        if umsg == windows::Win32::UI::WindowsAndMessaging::WM_KEYUP {
            // Toggle force_draw on Q keyup.
            if wparam == windows::Win32::UI::Input::KeyboardAndMouse::VK_Q.0 {
                self.force_draw = !self.force_draw;
            }
        }

        true // input should be passed to the game
    }

    fn draw(&mut self, ctx: &egui::Context, _shared_state: &mut SharedState, engine: &engine_api::Engine) {
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
