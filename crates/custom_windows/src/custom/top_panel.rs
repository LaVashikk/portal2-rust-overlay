use egui::Context;
use overlay_types::events::{OverlayEvent, push_event};
use portal2_sdk::Engine;

use crate::{SharedState, Window};

#[derive(Default)]
pub struct TopPanel;

impl Window for TopPanel {
    fn name(&self) -> &'static str { "Top Panel" }

    fn set_open(&mut self, _open: bool) {}
    fn is_open(&self) -> bool { true }

    fn is_should_render(&self, shared_state: &SharedState, _engine: &Engine) -> bool {
        shared_state.is_overlay_focused
    }

    fn draw(&mut self, ctx: &Context, _shared_state: &mut SharedState, _engine: &Engine) {
        let screen_rect = ctx.screen_rect();
        let painter = ctx.debug_painter();

        // Useful for user info
        let text = "[F3] Toggle Overlay • [Hold RMB] Free Look";
        let font_id = egui::FontId::proportional(24.0);
        let text_color = egui::Color32::WHITE;
        let shadow_color = egui::Color32::BLACK;
        let pos = egui::pos2(screen_rect.center().x, screen_rect.bottom() - 50.0);
        let anchor = egui::Align2::CENTER_BOTTOM;
        painter.text(pos + egui::vec2(2.0, 2.0), anchor, text, font_id.clone(), shadow_color);


        // Foreground text
        painter.text(pos, anchor, text, font_id, text_color);

        // Small watermark
        let watermark_text = "portal2-rust-overlay - Open-Source framework by laVashik";
        let watermark_font = egui::FontId::proportional(14.0);
        let watermark_color = egui::Color32::from_gray(180);
        let watermark_pos = egui::pos2(10.0, screen_rect.bottom() - 10.0);
        let watermark_anchor = egui::Align2::LEFT_BOTTOM;
        painter.text(watermark_pos + egui::vec2(1.0, 1.0), watermark_anchor, watermark_text, watermark_font.clone(), shadow_color);
        painter.text(watermark_pos, watermark_anchor, watermark_text, watermark_font, watermark_color);


        // Define the names of the windows that should go into the "Debug" dropdown.
        let debug_windows =[
            "Simple Window",
            "Engine API Demo",
            "ESP",
            "IVDebugOverlay Tester",
        ];

        egui::TopBottomPanel::top("overlay_top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.heading("P2 SDK");
                ui.separator();

                // Render debug windows in a dropdown menu
                ui.menu_button("⚙ Framework Debug", |ui| {
                    if let Some(windows) = crate::REGISTED_WINDOWS.get() {
                        for &win_name in windows {
                            if debug_windows.contains(&win_name) {
                                let mut is_open = ctx.data(|d| d.get_temp(egui::Id::new(win_name)).unwrap_or(false));

                                // Checkboxes look better in a dropdown menu
                                if ui.checkbox(&mut is_open, win_name).clicked() {
                                    push_event(OverlayEvent::SetWindowState(win_name, is_open));
                                }
                            }
                        }
                    }
                });

                ui.separator();

                // Render OTHER windows as tabs
                egui::ScrollArea::horizontal().id_salt("top_panel_tabs").show(ui, |ui| {
                    if let Some(windows) = crate::REGISTED_WINDOWS.get() {
                        for &win_name in windows {
                            // Skip framework UI and debug windows
                            if win_name == self.name() || debug_windows.contains(&win_name) {
                                continue;
                            }

                            let is_open = ctx.data(|d| d.get_temp(egui::Id::new(win_name)).unwrap_or(false));

                            if ui.selectable_label(is_open, win_name).clicked() {
                                push_event(OverlayEvent::SetWindowState(win_name, !is_open));
                            }
                        }
                    }
                });

                ui.separator();

                if ui.button("👁").on_hover_text("Open All").clicked() {
                    push_event(OverlayEvent::OpenAllWindows);
                }
                if ui.button("Ø").on_hover_text("Close All").clicked() {
                    push_event(OverlayEvent::CloseAllWindows);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌").on_hover_text("Close Overlay (F3)").clicked() {
                        push_event(OverlayEvent::ToggleOverlay);
                    }

                    ui.separator();

                    let mut debug_egui = ctx.data(|d| d.get_temp::<bool>(egui::Id::new("debug_egui")).unwrap_or(false));
                    if ui.toggle_value(&mut debug_egui, "🛠").on_hover_text("Toggle Egui Inspector").clicked() {
                        ctx.data_mut(|d| d.insert_temp(egui::Id::new("debug_egui"), debug_egui));
                    }

                    #[cfg(debug_assertions)]
                    {
                        let mut debug_on_hover = ctx.debug_on_hover();
                        if ui.toggle_value(&mut debug_on_hover, "📏").on_hover_text("Toggle Layout Debugger").clicked() {
                            ctx.set_debug_on_hover(debug_on_hover);
                        }
                    }
                });
            });
        });

        let debug_egui = ctx.data(|d| d.get_temp::<bool>(egui::Id::new("debug_egui")).unwrap_or(false));
        if debug_egui {
            egui::Window::new("Egui Inspection").show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });
            egui::Window::new("Egui Settings").show(ctx, |ui| {
                ctx.settings_ui(ui);
            });
            egui::Window::new("Egui Memory").show(ctx, |ui| {
                ctx.memory_ui(ui);
            });
        }
    }
}
