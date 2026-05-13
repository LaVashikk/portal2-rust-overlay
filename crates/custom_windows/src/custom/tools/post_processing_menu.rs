use std::ffi::CString;

use egui::{Context, ScrollArea, Slider, Ui, Vec2};
use overlay_types::{events::OverlayEvent, toasts};
use portal2_sdk::Engine;
use portal2_sdk::types::CEntityRespawnInfo;

use crate::{SharedState, Window};

#[derive(Default, PartialEq)]
enum PostProcessTab {
    #[default]
    ColorCorrection,
    MotionBlur,
    BloomAutoexposure,
}

/// Dedicated state for the Color Correction tab to keep the main struct clean
struct ColorCorrectionState {
    is_enabled: bool,
    weight: f32,
    selected_lut_index: usize,
    search_query: String,
    cached_luts: Vec<String>,
    needs_refresh: bool,
}

impl Default for ColorCorrectionState {
    fn default() -> Self {
        Self {
            is_enabled: false,
            weight: 1.0,
            selected_lut_index: 0,
            search_query: String::new(),
            cached_luts: Vec::new(),
            needs_refresh: true, // Force scan on first open
        }
    }
}

#[derive(Default)]
pub struct PostProcessingMenu {
    is_open: bool,
    current_tab: PostProcessTab,

    // Global tracker to prevent Entity Edict Limit crashes across all tabs
    last_hacky_respawn_time: f32,

    // Tab states
    cc: ColorCorrectionState,
}

impl Window for PostProcessingMenu {
    fn name(&self) -> &'static str { "Post Processing" }

    fn set_open(&mut self, open: bool) { self.is_open = open; }
    fn is_open(&self) -> bool { self.is_open }

    fn is_should_render(&self, shared_state: &SharedState, _engine: &Engine) -> bool {
        shared_state.is_overlay_focused
    }

    fn on_event(&mut self, event: &overlay_types::events::OverlayEvent, _shared_state: &mut SharedState) {
        match event {
            OverlayEvent::GameEvent(s) if s == "server_spawn" => {
                self.last_hacky_respawn_time = 0.0;
            }
            _ => {}
        }
    }

    fn draw(&mut self, ctx: &Context, shared_state: &mut SharedState, engine: &Engine) {
        let mut open = self.is_open;

        egui::Window::new(self.name())
            .open(&mut open)
            .resizable(true)
            .default_width(500.0)
            .default_height(350.0)
            .min_size(Vec2::new(450.0, 250.0)) // Prevent window from becoming too squished
            .show(ctx, |ui| {

                // --- TAB SELECTION BAR ---
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.current_tab, PostProcessTab::ColorCorrection, "Color Correction (LUT)");
                    ui.selectable_value(&mut self.current_tab, PostProcessTab::MotionBlur, "Motion Blur");
                    ui.selectable_value(&mut self.current_tab, PostProcessTab::BloomAutoexposure, "Bloom/Autoexposure");
                });

                ui.separator();

                // --- TAB CONTENT ---
                ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    match self.current_tab {
                        PostProcessTab::ColorCorrection => self.render_color_correction(ui, engine, shared_state),
                        PostProcessTab::MotionBlur => self.render_motion_blur(ui, engine, shared_state),
                        PostProcessTab::BloomAutoexposure => self.render_bloom(ui, engine, shared_state),
                    }
                });
            });

        self.is_open = open;
    }
}

impl PostProcessingMenu {
    // ==========================================
    // TAB: COLOR CORRECTION
    // ==========================================

    fn render_color_correction(&mut self, ui: &mut Ui, engine: &Engine, shared_state: &SharedState) {
        // Perform initial or manual scan for LUT files
        if self.cc.needs_refresh {
            self.scan_luts(shared_state);
            self.cc.needs_refresh = false;
        }

        ui.horizontal(|ui| {
            if ui.checkbox(&mut self.cc.is_enabled, "Override Color Correction").changed() {
                if self.cc.is_enabled {
                    self.apply_lut_preset(engine);
                } else {
                    // Disable the active custom color correction
                    engine.client().execute_client_cmd_unrestricted("ent_fire overlay_managed_cc Disable");
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").on_hover_text("Rescan virtual filesystem for .raw files").clicked() {
                    self.cc.needs_refresh = true;
                }
            });
        });

        ui.add_space(8.0);

        ui.add_enabled_ui(self.cc.is_enabled, |ui| {
            // Search bar
            ui.add_sized(
                [ui.available_width(), 20.0],
                egui::TextEdit::singleline(&mut self.cc.search_query)
                    .hint_text("🔍 Search...")
                    .desired_width(120.0)
            );

            ui.add_space(12.);

            ui.horizontal(|ui| {
                ui.label("Active LUT:");

                let combo_label = if self.cc.cached_luts.is_empty() {
                    "No LUTs found".to_string()
                } else {
                    self.cc.cached_luts[self.cc.selected_lut_index]
                        .split('/')
                        .last()
                        .unwrap_or("Unknown")
                        .to_string()
                };

                let mut lut_changed = false;

                // Right-to-Left alignment for a perfectly cohesive row
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {

                    // Copy Button
                    if ui.button("📋").on_hover_text("Copy LUT path").clicked() {
                        if !self.cc.cached_luts.is_empty() {
                            ui.ctx().copy_text(self.cc.cached_luts[self.cc.selected_lut_index].clone());
                        }
                    }

                    let query_lower = self.cc.search_query.to_lowercase();
                    let filtered_indices: Vec<usize> = self.cc.cached_luts
                        .iter()
                        .enumerate()
                        .filter(|(_, path)| query_lower.is_empty() || path.to_lowercase().contains(&query_lower))
                        .map(|(i, _)| i)
                        .collect();

                    // ComboBox
                    let combo_response = egui::ComboBox::from_id_salt("lut_selector")
                        .selected_text(combo_label)
                        .width(ui.available_width())
                        .height(400.0)
                        .show_ui(ui, |ui| {
                            if filtered_indices.is_empty() {
                                ui.colored_label(egui::Color32::DARK_GRAY, "No matches found");
                            } else {
                                for &i in &filtered_indices {
                                    let lut_path = &self.cc.cached_luts[i];
                                    let filename = lut_path.split('/').last().unwrap_or(lut_path);
                                    if ui.selectable_value(&mut self.cc.selected_lut_index, i, filename).clicked() {
                                        lut_changed = true;
                                    }
                                }
                            }
                        }).response;

                    // Mouse Wheel Scrolling logic
                    if combo_response.hovered() && !filtered_indices.is_empty() {
                        let scroll_y = ui.input(|i| i.raw_scroll_delta.y);

                        if scroll_y.abs() > 0.0 {
                            let current_filtered_pos = filtered_indices
                                .iter()
                                .position(|&idx| idx == self.cc.selected_lut_index)
                                .unwrap_or(0);

                            let new_filtered_pos = if scroll_y > 0.0 {
                                current_filtered_pos.saturating_sub(1)
                            } else {
                                (current_filtered_pos + 1).min(filtered_indices.len() - 1)
                            };

                            let new_index = filtered_indices[new_filtered_pos];

                            if self.cc.selected_lut_index != new_index {
                                self.cc.selected_lut_index = new_index;
                                lut_changed = true;
                            }

                            // Consume the scroll event
                            ui.input_mut(|i| {
                                i.raw_scroll_delta = egui::Vec2::ZERO;
                                i.smooth_scroll_delta = egui::Vec2::ZERO;
                            });
                        }
                    }
                });

                // Apply changes if needed
                if lut_changed && self.cc.is_enabled {
                    self.apply_lut_preset(engine);
                }
            });

            ui.add_space(8.0);

            // --- WEIGHT SLIDER ---
            ui.horizontal(|ui| {
                ui.label("Intensity:");
                let response = ui.add_sized(
                    [ui.available_width(), 20.0],
                    Slider::new(&mut self.cc.weight, 0.0..=1.0)
                        .trailing_fill(true)
                        .smart_aim(true)
                );

                if response.changed() && self.cc.is_enabled {
                    self.apply_lut_preset(engine);
                }
            });
        });
    }

    /// Scans the physical directories mapped in the virtual filesystem for valid 96KB .raw LUTs
    fn scan_luts(&mut self, shared_state: &SharedState) {
        self.cc.cached_luts.clear();

        let fs = &shared_state.valve_fs;
        if let Some(dirs) = fs.search_path_dirs().get("game") {
            for dir in dirs {
                let target_path = fs.root_path().join(dir).join("materials/correction");

                if let Ok(entries) = std::fs::read_dir(target_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();

                        if path.extension().and_then(|e| e.to_str()) == Some("raw") {
                            // Validate the 96KB size requirement for Source Engine LUTs
                            if let Ok(meta) = path.metadata() {
                                if meta.len() == 96 * 1024 { // 98304 bytes
                                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                        self.cc.cached_luts.push(format!("materials/correction/{}", name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.cc.cached_luts.sort();
        self.cc.cached_luts.dedup(); // Remove duplicates from overlapping mount paths

        if self.cc.selected_lut_index >= self.cc.cached_luts.len() {
            self.cc.selected_lut_index = 0;
        }
    }

    /// Prepares parameters and triggers the universal entity respawner for Color Correction
    fn apply_lut_preset(&mut self, engine: &Engine) {
        if self.cc.cached_luts.is_empty() {
            return;
        }

        let lut_path = self.cc.cached_luts[self.cc.selected_lut_index].clone();
        let weight_str = format!("{:.3}", self.cc.weight);
        let properties =[
            ("filename", lut_path.as_str()),
            ("spawnflags", "1"),
            ("exclusive", "1"),
            ("maxweight", weight_str.as_str()),
            ("minfalloff", "-1.0"),
            ("maxfalloff", "-1.0"),
            ("fadeInDuration", "0.0"),
            ("fadeOutDuration", "0.0"),
        ];

        self.hacky_respawn_custom_entity(engine, "color_correction", "overlay_managed_cc", &properties);
        engine.client().execute_client_cmd_unrestricted("ent_fire overlay_managed_cc Enable");
    }

    // ==========================================
    // TAB: MOTION BLUR
    // ==========================================

    fn render_motion_blur(&self, ui: &mut Ui, engine: &Engine, shared_state: &mut SharedState) {
        let cvars =[
            "mat_motion_blur_enabled",
            "mat_motion_blur_forward_enabled",
            "mat_motion_blur_falling_min",
            "mat_motion_blur_falling_max",
            "mat_motion_blur_falling_intensity",
            "mat_motion_blur_roll_intensity",
            "mat_motion_blur_rotation_intensity",
            "mat_motion_blur_strength",
        ];

        ui.horizontal_wrapped(|ui| {
            draw_cvar_checkbox(ui, engine, cvars[0], "Blur Enabled");
            ui.add_space(16.0);
            draw_cvar_checkbox(ui, engine, cvars[1], "Blur Forward Enabled");
        });

        ui.add_space(12.0);

        egui::Grid::new("motion_blur_grid")
            .num_columns(2)
            .spacing(Vec2::new(32.0, 8.0))
            .show(ui, |ui| {
                draw_cvar_slider_row(ui, engine, cvars[2], "Falling Min", 0.0..=50.0);
                draw_cvar_slider_row(ui, engine, cvars[3], "Falling Max", 0.0..=50.0);
                draw_cvar_slider_row(ui, engine, cvars[4], "Falling Intensity", 0.0..=5.0);
                draw_cvar_slider_row(ui, engine, cvars[5], "Roll Intensity", 0.0..=5.0);
                draw_cvar_slider_row(ui, engine, cvars[6], "Rotation Intensity", 0.0..=5.0);
                draw_cvar_slider_row(ui, engine, cvars[7], "Strength", 0.0..=5.0);
            });

        self.render_copy_buttons(ui, engine, shared_state, &cvars);
    }

    // ==========================================
    // TAB: BLOOM / AUTOEXPOSURE
    // ==========================================

    fn render_bloom(&self, ui: &mut Ui, engine: &Engine, shared_state: &mut SharedState) {
        let cvars =[
            "mat_force_bloom",
            "mat_disable_bloom",
            "mat_bloomscale",
            "mat_bloom_scalefactor_scalar",
            "mat_bloomamount_rate",
            "mat_autoexposure_max",
            "mat_autoexposure_max_multiplier",
            "mat_autoexposure_min",
            "mat_hdr_uncap_autoexposure",
            "mat_accelerate_adjust_exposure_down",
        ];

        ui.horizontal_wrapped(|ui| {
            draw_cvar_checkbox(ui, engine, cvars[0], "Force Bloom");
            ui.add_space(16.0);
            draw_cvar_checkbox(ui, engine, cvars[1], "Disable Bloom");
            ui.add_space(16.0);
            draw_cvar_checkbox(ui, engine, cvars[8], "Uncap Autoexposure");
        });

        ui.add_space(12.0);

        egui::Grid::new("bloom_grid")
            .num_columns(2)
            .spacing(Vec2::new(32.0, 8.0))
            .show(ui, |ui| {
                draw_cvar_slider_row(ui, engine, cvars[2], "Bloom Scale", 0.0..=5.0);
                draw_cvar_slider_row(ui, engine, cvars[3], "Bloom Scalefactor Scalar", 0.0..=5.0);
                draw_cvar_slider_row(ui, engine, cvars[4], "Bloom Rate", 0.0..=2.0);
                draw_cvar_slider_row(ui, engine, cvars[5], "Autoexposure Max", 0.0..=5.0);
                draw_cvar_slider_row(ui, engine, cvars[6], "Autoexposure Max Multiplier", 0.0..=5.0);
                draw_cvar_slider_row(ui, engine, cvars[7], "Autoexposure Min", 0.0..=2.0);
                draw_cvar_slider_row(ui, engine, cvars[9], "Accelerate Exposure Down", 0.0..=20.0);
            });

        self.render_copy_buttons(ui, engine, shared_state, &cvars);
    }

    // ==========================================
    // SHARED / UTILITY METHODS
    // ==========================================

    fn render_copy_buttons(&self, ui: &mut Ui, engine: &Engine, _shared_state: &mut SharedState, cvars: &[&str]) {
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(4.0);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("📋 Copy Settings").on_hover_text("Copy the ConVars of this tab to clipboard").clicked() {
                let mut result = String::new();
                for &cvar_name in cvars {
                    if let Some(cvar) = engine.cvar_system().find_var(cvar_name) {
                        result.push_str(&format!("{} {}; ", cvar_name, cvar.get_string()));
                    }
                }
                ui.ctx().copy_text(result);
                toasts::success("Settings copied to clipboard!", 2000);
            }
        });
    }

    /// Generic powerful method to inject ANY entity into the map via a VMF block.
    /// It safeguards against double-spawning in the same engine tick.
    fn hacky_respawn_custom_entity(&mut self, engine: &Engine, classname: &str, targetname: &str, properties: &[(&str, &str)]) {
        // Prevent double updates within a single game tick!
        // This stops "CreateEdict failed" engine crashes.
        let time_stamp = engine.client().get_last_time_stamp();
        if time_stamp <= self.last_hacky_respawn_time {
            return;
        }
        self.last_hacky_respawn_time = time_stamp;

        // Auto-fetch local player's origin for entity placement
        let mut origin_str = String::from("0 0 0");
        if let Some(player) = engine.entities().find_by_classname(None, "player") {
            let origin = player.get_origin();
            origin_str = format!("{:.2} {:.2} {:.2}", origin.x, origin.y, origin.z);
        }

        // Programmatically build the VMF entity block
        let mut ent_text = String::with_capacity(512);
        ent_text.push_str("entity\n{\n");
        ent_text.push_str(&format!("\t\"classname\" \"{}\"\n", classname));
        ent_text.push_str(&format!("\t\"targetname\" \"{}\"\n", targetname));
        ent_text.push_str(&format!("\t\"origin\" \"{}\"\n", origin_str));
        ent_text.push_str("\t\"hammerid\" \"9999999\"\n");

        for (k, v) in properties {
            ent_text.push_str(&format!("\t\"{}\" \"{}\"\n", k, v));
        }
        ent_text.push_str("}\n");

        // Execute via Source Engine network strings
        if let Ok(c_ent_text) = CString::new(ent_text) {
            let mut info = CEntityRespawnInfo {
                hammer_id: 9999999,
                ent_text: c_ent_text.as_ptr(),
            };
            engine.server_tools().respawn_entities_with_edits(std::slice::from_mut(&mut info));
        }
    }
}

// --- Global Helpers ---

fn execute(engine: &Engine, cmd: &str) {
    engine.client().execute_client_cmd_unrestricted(cmd);
}

fn draw_cvar_checkbox(ui: &mut Ui, engine: &Engine, cvar_name: &str, label: &str) {
    if let Some(cvar) = engine.cvar_system().find_var(cvar_name) {
        let mut is_checked = cvar.get_int() != 0;

        if ui.checkbox(&mut is_checked, label).on_hover_text(cvar_name).changed() {
            let new_value = if is_checked { 1 } else { 0 };
            execute(engine, &format!("{} {}", cvar_name, new_value));
        }
    } else {
        ui.add_enabled(false, egui::Checkbox::new(&mut false, label))
            .on_hover_text(format!("ConVar '{}' not found", cvar_name));
    }
}

fn draw_cvar_slider_row(ui: &mut Ui, engine: &Engine, cvar_name: &str, label: &str, range: std::ops::RangeInclusive<f32>) {
    ui.label(label).on_hover_text(cvar_name);

    if let Some(cvar) = engine.cvar_system().find_var(cvar_name) {
        let mut value = cvar.get_float();
        let slider = Slider::new(&mut value, range).show_value(true).trailing_fill(true);

        if ui.add_sized([ui.available_width(), 20.0], slider).changed() {
            execute(engine, &format!("{} {}", cvar_name, value));
        }
    } else {
        let mut dummy = 0.0;
        let slider = Slider::new(&mut dummy, range).show_value(true);
        ui.add_enabled(false, slider).on_hover_text(format!("ConVar '{}' not found", cvar_name));
    }

    ui.end_row();
}
