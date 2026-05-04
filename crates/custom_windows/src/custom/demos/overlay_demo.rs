use egui::{Context, Slider};
use portal2_sdk::{Engine, types::{Vector, QAngle}};

use crate::{SharedState, Window};

pub struct DebugOverlayDemo {
    is_open: bool,

    // Shared parameters
    duration: f32,
    color: [u8; 4],
    distance: f32,

    // Box specific
    box_size: f32,

    // Sphere specific
    sphere_radius: f32,
    sphere_theta: i32,
    sphere_phi: i32,

    // Text specific
    world_text: String,
    screen_text: String,
    screen_x: f32,
    screen_y: f32,
}

impl Default for DebugOverlayDemo {
    fn default() -> Self {
        Self {
            is_open: false,
            duration: 5.0,
            color: [255, 100, 50, 200],
            distance: 200.0,

            box_size: 16.0,

            sphere_radius: 32.0,
            sphere_theta: 10,
            sphere_phi: 10,

            world_text: "Hello 3D World!".into(),
            screen_text: "Hello 2D Screen!".into(),
            screen_x: 0.5,
            screen_y: 0.5,
        }
    }
}

impl Window for DebugOverlayDemo {
    fn name(&self) -> &'static str { "IVDebugOverlay Tester" }
    fn set_open(&mut self, open: bool) { self.is_open = open; }
    fn is_open(&self) -> bool { self.is_open }

    fn is_should_render(&self, shared_state: &SharedState, _engine: &Engine) -> bool {
        shared_state.is_overlay_focused
    }

    fn draw(&mut self, ctx: &Context, shared_state: &mut SharedState, engine: &Engine) {
        // --- Continuous Background Rendering ---

        let target_data = get_point_in_front(engine, self.distance);
        let debug = engine.debug_overlay();

        // --- UI Rendering ---
        if !shared_state.is_overlay_focused {
            return;
        }

        let mut open = self.is_open;
        egui::Window::new(self.name())
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Duration:");
                    ui.add(Slider::new(&mut self.duration, 0.0..=20.0).suffix("s"));
                });
                ui.horizontal(|ui| {
                    ui.label("Distance from player:");
                    ui.add(Slider::new(&mut self.distance, 50.0..=1000.0).suffix(" units"));
                });
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    ui.color_edit_button_srgba_unmultiplied(&mut self.color);
                });

                ui.separator();

                if let Some((eye_pos, target_pos)) = target_data {
                    let [r, g, b, a] = self.color;
                    let (ri, gi, bi, ai) = (r as i32, g as i32, b as i32, a as i32);

                    ui.collapsing("1. Add Box Overlay", |ui| {
                        ui.add(Slider::new(&mut self.box_size, 1.0..=100.0).text("Size"));
                        if ui.button("Spawn Box").clicked() {
                            let mins = Vector::new(-self.box_size, -self.box_size, -self.box_size);
                            let maxs = Vector::new(self.box_size, self.box_size, self.box_size);
                            debug.add_box_overlay(&target_pos, &mins, &maxs, &QAngle::default(), ri, gi, bi, ai, self.duration);
                        }
                    });

                    ui.collapsing("2. Add Sphere Overlay", |ui| {
                        ui.add(Slider::new(&mut self.sphere_radius, 1.0..=100.0).text("Radius"));
                        ui.add(Slider::new(&mut self.sphere_theta, 3..=50).text("Theta (Segments)"));
                        ui.add(Slider::new(&mut self.sphere_phi, 3..=50).text("Phi (Rings)"));
                        if ui.button("Spawn Sphere").clicked() {
                            debug.add_sphere_overlay(&target_pos, self.sphere_radius, self.sphere_theta, self.sphere_phi, ri, gi, bi, ai, self.duration);
                        }
                    });

                    ui.collapsing("3. Add Line Overlay", |ui| {
                        if ui.button("Spawn Line (Eye to Target)").clicked() {
                            // true = ignore depth (renders through walls)
                            debug.add_line_overlay(&eye_pos, &target_pos, ri, gi, bi, true, self.duration);
                        }
                    });

                    ui.collapsing("4. Add Text Overlay (broken?)", |ui| {
                        ui.text_edit_singleline(&mut self.world_text);
                        if ui.button("Spawn World Text").clicked() {
                            debug.add_text_overlay(&target_pos, self.duration, &self.world_text);
                        }
                    });

                    ui.collapsing("5. Add Screen Text Overlay (2D)", |ui| {
                        // Usually 0.0 to 1.0 representing screen percentage
                        ui.add(Slider::new(&mut self.screen_x, 0.0..=1.0).text("X Position"));
                        ui.add(Slider::new(&mut self.screen_y, 0.0..=1.0).text("Y Position"));
                        ui.text_edit_singleline(&mut self.screen_text);
                        if ui.button("Spawn Screen Text").clicked() {
                            debug.add_screen_text_overlay(self.screen_x, self.screen_y, self.duration, ri, gi, bi, ai, &self.screen_text);
                        }
                    });

                    ui.separator();

                    ui.add_space(8.0);

                    if ui.button("Clear All Engine Overlays").clicked() {
                        debug.clear_all_overlays();
                    }
                } else {
                    ui.colored_label(egui::Color32::RED, "Waiting for local player to spawn...");
                }
            });

        self.is_open = open;
    }
}

/// Helper function to calculate a point in the 3D space relative to the player's view
fn get_point_in_front(engine: &Engine, distance: f32) -> Option<(Vector, Vector)> {
    let server_tools = engine.server_tools();
    if let Some((eye_pos, angles)) = server_tools.get_player_position(None) {
        let forward = angles.to_forward_vector();

        // Calculate the target position
        let target_pos = eye_pos + (forward * distance);

        Some((eye_pos, target_pos))
    } else {
        None
    }
}
