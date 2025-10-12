use crate::{SharedState, Window};
use super::{FormAction, WidgetForm};
use egui::{Align2, Color32, Stroke};
use engine_api::Engine;

const BUG_ICON: &str = "❗";

#[derive(Debug)]
pub struct BugReportWin {
    form: WidgetForm,
    is_modal_open: bool,
}

impl BugReportWin {
    pub fn new(config_path: &str) -> Self {
        Self {
            form: WidgetForm::new(config_path),
            is_modal_open: false,
        }
    }

    fn save_form_results(&self, engine: &Engine) -> Result<(), String> {
        let mut extra_data = std::collections::BTreeMap::new();  // TODO: just for testing purposes
        let mut current_angles = engine_api::types::QAngle::default();
        engine.client().get_view_angles(&mut current_angles);
        extra_data.insert("player_pos".to_string(), serde_json::json!(format!("Vector({}, {}, {})", current_angles.x, current_angles.y, current_angles.z)));
        extra_data.insert("test".to_string(), serde_json::json!("passed!"));

        self.form.save_results(engine, Some(extra_data))
    }

    fn close_window(&mut self, shared_state: &mut SharedState) {
        self.is_modal_open = false;
        shared_state.is_overlay_focused = false;
        self.form.reset_state();
    }

    fn draw_button(&mut self, ctx: &egui::Context) {
        const PADDING: f32 = 25.0;
        const BUTTON_SIZE: f32 = 70.0;
        let button_size_vec = egui::vec2(BUTTON_SIZE, BUTTON_SIZE);

        egui::Area::new("bug_report_btn".into())
            .fixed_pos(ctx.screen_rect().right_bottom() - button_size_vec - egui::vec2(PADDING, PADDING))
            .show(ctx, |ui|
        {
            let (rect, response) = ui.allocate_exact_size(button_size_vec, egui::Sense::click());
            let is_hovered = response.hovered();

            let bg_fill = if is_hovered {
                Color32::from_rgba_unmultiplied(60, 60, 60, 220)
            } else {
                Color32::from_rgba_unmultiplied(35, 35, 35, 200)
            };

            let stroke = if is_hovered {
                Stroke::new(2.0, Color32::WHITE)
            } else {
                Stroke::new(2.0, Color32::from_gray(150))
            };

            let icon_color = if is_hovered { Color32::WHITE } else { Color32::from_gray(200) };

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                painter.circle(
                    rect.center(),
                    rect.width() / 2.0,
                    bg_fill,
                    stroke,
                );
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    BUG_ICON,
                    egui::FontId::proportional(BUTTON_SIZE * 0.6),
                    icon_color,
                );
            }

            if is_hovered && response.clicked() {
                self.is_modal_open = true;
            }
            response.on_hover_text("Report a Bug or suggest an idea");
        });
    }
}

impl Window for BugReportWin {
    fn name(&self) -> &'static str { "Bug Report" }

    fn is_should_render(&self, shared_state: &SharedState, engine: &Engine) -> bool {
        // The window should only be rendered when the game is paused to show the button.
        engine.client().is_paused() && engine.client().is_in_game() && !shared_state.surver_is_opened
    }

    fn draw(&mut self, ctx: &egui::Context, shared_state: &mut SharedState, engine: &Engine) {
        if self.is_modal_open {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.close_window(shared_state);
            }

            shared_state.is_overlay_focused = true;
            let action = self.form.draw_modal_window(ctx, engine, true);
            match action {
                FormAction::Submitted => {
                    // The Submit button was clicked
                    if let Err(e) = self.save_form_results(engine) {
                        log::error!("Failed to save bug report: {}", e);
                    }

                    self.close_window(shared_state);
                }
                FormAction::Closed => {
                    self.close_window(shared_state);
                }
                FormAction::None => {}
            }
        } else {
            self.draw_button(ctx);
        }
    }

    fn on_raw_input(&mut self, umsg: u32, _wparam: u16) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{WM_CHAR, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP};
        if !self.is_modal_open { return true; }

        match umsg {
            WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP | WM_CHAR => false,
            _ => true
        }
    }

    fn toggle(&mut self) { /* Controlling via a button in the UI */ }
    fn is_open(&self) -> bool { true }
}
