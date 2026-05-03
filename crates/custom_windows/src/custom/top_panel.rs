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
        egui::TopBottomPanel::top("overlay_top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.heading("P2 SDK");
                ui.separator();

                egui::ScrollArea::horizontal().id_salt("top_panel_tabs").show(ui, |ui| {
                    if let Some(windows) = crate::custom::REGISTED_WINDOWS.get() {
                        for &win_name in windows {
                            if win_name == self.name() || win_name == "Overlay Text" {
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
