use egui::{Align2, Color32, Context, Slider, Vec2, pos2};
use portal2_sdk::{Engine, types::{QAngle, Vector}};

use crate::{SharedState, Window};

pub struct Esp {
    is_open: bool,
    show_egui_text: bool,
    show_native_boxes: bool,
    show_snaplines: bool,
    max_distance: f32,
}

impl Default for Esp {
    fn default() -> Self {
        Self {
            is_open: false,
            show_egui_text: true,
            show_native_boxes: false,
            show_snaplines: false,
            max_distance: 1500.0,
        }
    }
}

impl Window for Esp {
    fn name(&self) -> &'static str { "ESP" }
    fn set_open(&mut self, open: bool) { self.is_open = open; }
    fn is_open(&self) -> bool { self.is_open }

    // The ESP should render when it's enabled
    fn is_should_render(&self, _shared_state: &SharedState, _engine: &Engine) -> bool {
        true
    }

    fn draw(&mut self, ctx: &Context, shared_state: &mut SharedState, engine: &Engine) {
        // Draw the Configuration Window only when overlay is focused
        if shared_state.is_overlay_focused {
            let mut open = self.is_open;
            egui::Window::new(self.name())
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.checkbox(&mut self.show_native_boxes, "Native 3D Boxes (DebugOverlay)");
                    ui.checkbox(&mut self.show_egui_text, "Egui World-To-Screen Text");
                    ui.checkbox(&mut self.show_snaplines, "Egui Snaplines");

                    ui.add_space(8.0);
                    ui.add(Slider::new(&mut self.max_distance, 100.0..=5000.0).text("Max Distance"));
                });
            self.is_open = open;
        }

        // Draw the actual ESP overlay
        if self.is_open {
            self.render_esp(ctx, engine);
        }
    }
}

impl Esp {
    fn render_esp(&self, ctx: &Context, engine: &Engine) {
        if !self.show_egui_text && !self.show_native_boxes && !self.show_snaplines {
            return;
        }

        let painter = ctx.debug_painter();
        let screen_rect = ctx.screen_rect();
        let screen_center_bottom = screen_rect.center_bottom();

        let ents = engine.entities();
        let local_player = ents.find_by_classname(None, "player");
        let local_player_origin = local_player.map(|p| p.get_origin()).unwrap_or_default();

        let debug_overlay = engine.debug_overlay();

        for ent in ents.iter() {
            let classname = ent.get_classname();

            // Filter out junk entities
            if classname == "player" || classname == "worldspawn" || classname.is_empty() {
                continue;
            }

            // To prevent screen clutter, only show specific entity types
            if !classname.starts_with("prop_")  {
                continue;
            }

            let origin = ent.get_origin();
            let dist = local_player_origin.distance(&origin);

            if dist > self.max_distance || dist <= 0.0 {
                continue;
            }

            // --- Native Engine 3D Box ---
            if self.show_native_boxes {
                // We use arbitrary small bounds. In a real scenario, you would extract
                // the collision bounds from the entity using ICollideable.
                let mins = Vector::new(-12.0, -12.0, -12.0);
                let maxs = Vector::new(12.0, 12.0, 12.0);
                let angles = QAngle::default();

                // r, g, b, a, duration
                // A duration of 0.0 means it lives for exactly 1 tick/frame
                debug_overlay.add_box_overlay(&origin, &mins, &maxs, &angles, 255, 50, 50, 64, 0.03);
            }

            // --- gui World-To-Screen Rendering ---
            if self.show_egui_text || self.show_snaplines {
                // dbg!("world_to_screen");
                if let Some(screen_pos) = debug_overlay.world_to_screen(&origin) {
                    // Source Engine screen coordinates -> Egui logical points
                    // We divide by pixels_per_point to ensure it scales correctly if UI scaling is active
                    let e_pos = pos2(
                        screen_pos.x / ctx.pixels_per_point(),
                        screen_pos.y / ctx.pixels_per_point(),
                    );

                    // Draw Snaplines (Lines from bottom of screen to the entity)
                    if self.show_snaplines {
                        painter.line_segment([screen_center_bottom, e_pos],
                            (1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 30)), // Faint white line
                        );
                    }

                    // Draw Text
                    if self.show_egui_text {
                        let text = format!("{} [{:.0}u]", classname, dist);

                        // Draw black shadow for readability against bright game backgrounds
                        painter.text(
                            e_pos + Vec2::new(1.0, 1.0),
                            Align2::CENTER_CENTER,
                            &text,
                            egui::FontId::proportional(12.0),
                            Color32::BLACK,
                        );

                        // Draw main colored text
                        painter.text(
                            e_pos,
                            Align2::CENTER_CENTER,
                            &text,
                            egui::FontId::proportional(12.0),
                            Color32::GREEN,
                        );
                    }
                }
            }
        }
    }
}
