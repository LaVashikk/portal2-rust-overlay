use std::collections::HashSet;
use std::path::{Path, PathBuf};

use egui::{Color32, Context, ScrollArea, TextEdit, Ui, Vec2};
use overlay_types::events::{OverlayEvent, push_event};
use overlay_types::toasts;
use portal2_sdk::Engine;
use portal2_sdk::types::MaskFlags;
use source_vmt::{Value, Vmt};
use source_fs::{FileSystem, providers::DummyVpk};

use crate::{SharedState, Window};

pub struct MaterialInspector {
    is_open: bool,
    realtime_preview: bool,

    current_material_name: String,
    current_vmt: Option<Vmt>,
    current_file_path: Option<PathBuf>,

    // Stores raw bytes of the original file to perfectly revert previews
    backup_vmt_bytes: Option<Vec<u8>>,
    new_prop_key: String,
    new_prop_value: String,

    // file_dialog: FileDialog,
    disabled_properties: HashSet<String>,

    error_message: Option<String>,
    mat_sys: source_vmt::MaterialSystem<DummyVpk>,
}

impl MaterialInspector {
    pub fn new(shared_state: &SharedState) -> Self {
        let mat_sys = source_vmt::MaterialSystem::new(shared_state.valve_fs.clone())
            .with_search_path("game")
            .prioritize_vpks(true);

        Self {
            is_open: false,
            realtime_preview: false,
            current_material_name: String::new(),
            current_vmt: None,
            current_file_path: None,
            backup_vmt_bytes: None,
            new_prop_key: String::new(),
            new_prop_value: String::new(),
            // file_dialog: FileDialog::new(),
            disabled_properties: HashSet::new(),
            error_message: None,
            mat_sys,
        }
    }

    // ==========================================
    // UI DRAWING METHODS
    // ==========================================

    fn draw_toolbar(&mut self, ui: &mut Ui, engine: &Engine) {
        ui.horizontal(|ui| {
            if ui.button("🎯 Pick Material")
                .on_hover_text("Trace ray from eyes and grab world material")
                .clicked()
            {
                self.pick_material_from_crosshair(engine);
            }

            ui.separator();

            // Draw label before layout changes to prevent overlapping
            ui.label("Target:");

            // Allocate checkbox to the right, fill the rest with TextEdit
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.realtime_preview, "Real-time Preview")
                    .on_hover_text("Automatically refresh the material in-game on every change");

                ui.separator();

                ui.add_enabled_ui(false, |ui| {
                    ui.add(TextEdit::singleline(&mut self.current_material_name).desired_width(f32::INFINITY));
                });
            });
        });
    }

    fn draw_footer(&mut self, ui: &mut Ui, engine: &Engine) {
        ui.horizontal(|ui| {
            let has_path = self.current_file_path.is_some();
            let has_vmt = self.current_vmt.is_some();

            // Calculate exactly 25% width for each button
            let spacing = ui.spacing().item_spacing.x;
            let button_width = (ui.available_width() - spacing * 3.0) / 3.0;

            ui.add_enabled_ui(has_vmt, |ui| {
                if ui.add_sized([button_width, 25.0], egui::Button::new("👁 Preview")).clicked() {
                    self.preview_material();
                }
            });

            let save_response = ui.add_enabled_ui(has_path, |ui| {
                if ui.add_sized([button_width, 25.0], egui::Button::new("💾 Save")).clicked() {
                    self.save_and_apply(&self.current_file_path.clone().unwrap(), false);
                }
                // if ui.add_sized([button_width, 25.0], egui::Button::new("💾 Save as")).clicked() {
                //     self.file_dialog.save_file();
                // }
                if ui.add_sized([button_width, 25.0], egui::Button::new("✅ Save & Apply")).clicked() {
                    self.save_and_apply(&self.current_file_path.clone().unwrap(), true);
                }
            }).response;

            // if let Some(path) = self.file_dialog.take_picked() {
            //     log::info!("saving to {}", path.display());
            // }

            // Display warning as a tooltip instead of rendering raw path text
            if !has_path && has_vmt {
                save_response.on_hover_text("Cannot save: file is inside a VPK or missing.");
            }
        });
    }

    fn draw_vmt_block(
        ui: &mut Ui,
        properties: &mut indexmap::IndexMap<source_vmt::VmtKey, Vec<Value>>,
        block_name: &str,
        fs: &FileSystem<DummyVpk>,
        keys_to_remove: &mut Vec<String>,
        disabled_keys: &mut HashSet<String>,
        shift_held: bool,
    ) -> bool {
        let mut any_changed = false;

        egui::Grid::new(format!("grid_{}", block_name))
            .num_columns(2)
            .min_col_width(120.0)
            .spacing(Vec2::new(16.0, 8.0))
            .striped(true)
            .show(ui, |ui| {
                for (k, v_list) in properties.iter_mut() {
                    for v in v_list.iter_mut() {
                        if let Value::Str(s) = v {
                            let is_disabled = disabled_keys.contains(k.as_str());

                            ui.horizontal(|ui| {
                               if block_name == "root" {
                                    if shift_held {
                                        let trash_btn = egui::Button::new(
                                            egui::RichText::new("🗑").color(Color32::RED)
                                        ).frame(false);

                                        if ui.add(trash_btn).on_hover_text("Delete property").clicked() {
                                            keys_to_remove.push(k.to_string());
                                        }
                                    } else {
                                        let icon_color = if is_disabled { Color32::DARK_GRAY } else { Color32::LIGHT_GRAY };
                                        let eye_btn = egui::Button::new(
                                            egui::RichText::new("👁").color(icon_color)
                                        ).frame(false);

                                        if ui.add(eye_btn).on_hover_text("Toggle visibility (Hold SHIFT to delete)").clicked() {
                                            if is_disabled {
                                                disabled_keys.remove(k.as_str());
                                            } else {
                                                disabled_keys.insert(k.to_string());
                                            }
                                            any_changed = true; // Триггерим превью при скрытии/показе слоя
                                        }
                                    }
                                }

                                let label_text = egui::RichText::new(k.to_string());
                                if is_disabled {
                                    ui.label(label_text.color(Color32::DARK_GRAY).strikethrough());
                                } else {
                                    ui.label(label_text);
                                }
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Strictly allocate 65px block on the right for validation icons
                                ui.allocate_ui_with_layout(
                                    Vec2::new(25.0, ui.available_height()),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        if s.contains('/') || s.contains('\\') {
                                            let is_valid = fs.find_asset(s, "materials/", ".vtf", "game").is_some()
                                                || fs.find_asset(s, "materials/", ".vtf", "mod").is_some();

                                            if is_valid {
                                                ui.colored_label(if is_disabled {Color32::DARK_GRAY} else {Color32::GREEN}, "✔");
                                            } else {
                                                ui.colored_label(Color32::RED, "❌").on_hover_text("Texture not found");
                                            }
                                        }
                                    }
                                );

                                // TextEdit fills exactly the remaining space, ensuring perfectly aligned right edges
                                let response = ui.add(TextEdit::singleline(s).desired_width(f32::INFINITY));
                                if response.changed() {
                                    any_changed = true;
                                }
                            });

                            ui.end_row();
                        }
                    }
                }
            });

        // Draw nested proxies/blocks
        for (k, v_list) in properties.iter_mut() {
            let len = v_list.len();
            for (idx, v) in v_list.iter_mut().enumerate() {
                if let Value::Obj(map) = v {
                    ui.add_space(8.0);

                    let title = if len > 1 {
                        format!("{} [{}]", k, idx)
                    } else {
                        k.to_string()
                    };

                    egui::CollapsingHeader::new(egui::RichText::new(title).strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            let block_id = format!("{}_{}_{}", block_name, k, idx);
                            any_changed |= Self::draw_vmt_block(ui, map, &block_id, fs, keys_to_remove, disabled_keys, shift_held);
                        });
                }
            }
        }

        any_changed
    }

    // ==========================================
    // BUSINESS LOGIC & ENGINE COMMANDS
    // ==========================================

    /// Creates a "clean" copy of the VMT with all hidden properties removed.
    fn get_exportable_vmt(&self) -> Option<Vmt> {
        let mut vmt = self.current_vmt.clone()?;
        for key in &self.disabled_properties {
            vmt.remove(key);
        }
        Some(vmt)
    }

    fn preview_material(&mut self) {
        if let (Some(export_vmt), Some(path)) = (self.get_exportable_vmt(), &self.current_file_path) {
            if self.backup_vmt_bytes.is_none() {
                if let Ok(bytes) = std::fs::read(path) {
                    self.backup_vmt_bytes = Some(bytes);
                }
            }

            if let Ok(serialized_vmt) = export_vmt.to_string() {
                if std::fs::write(path, serialized_vmt).is_ok() {
                    let cmd = format!("mat_reloadmaterial {}", self.current_material_name);
                    push_event(OverlayEvent::EngineCommand(cmd));
                } else {
                    toasts::error("Failed to write preview to disk.", 3000);
                }
            }
        }
    }

    /// Reverts the file on disk to its exact original byte state.
    fn revert_preview(&mut self) {
        if let (Some(backup), Some(path)) = (self.backup_vmt_bytes.take(), &self.current_file_path) {
            let _ = std::fs::write(path, backup);
        }
    }

    fn pick_material_from_crosshair(&mut self, engine: &Engine) {
        // Must revert any active preview before picking a new material
        self.revert_preview();

        self.error_message = None;
        self.current_vmt = None;
        self.current_file_path = None;
        self.current_material_name.clear();
        self.disabled_properties.clear();

        let e = engine.entities();
        let local_player = e.find_by_classname(None, "player");

        let server_tools = engine.server_tools();
        if let Some((pos, angles)) = server_tools.get_player_position(None) {
            let forward = angles.to_forward_vector();
            let end_pos = pos + (forward * 8192.0);

            let trace = engine.engine_trace().line_trace(pos, end_pos, MaskFlags::SOLID, local_player.as_deref());

            if trace.did_hit_world() {
                let surf_name = trace.get_surface_name();

                if surf_name.is_empty() || surf_name == "**studio**" {
                    self.error_message = Some("Invalid surface hit".into());
                    return;
                }

                self.current_material_name = surf_name.to_string();
                self.load_material();
                self.preview_material();
            } else {
                self.error_message = Some("Did not hit world geometry.".into());
                toasts::error("Did not hit world geometry", 3000);
            }
        } else {
            self.error_message = Some("Could not get player position.".into());
            toasts::error("Failed to get player position", 3000);
        }
    }

    fn load_material(&mut self) {
        let mat_name = &self.current_material_name;
        if let Ok(vmt_arc) = self.mat_sys.get_material(mat_name) {
            self.current_vmt = Some((*vmt_arc).clone());
            toasts::success(format!("Loaded: {}", mat_name), 3000);

            self.current_file_path = self.mat_sys.fs.find_asset(mat_name, "materials/", ".vmt", "game");
        } else {
            self.error_message = Some(format!("Material '{}' failed to parse or missing.", mat_name));
            toasts::error("Failed to load material", 3000);
        }
    }

    fn save_and_apply(&mut self, path: &Path, should_apply: bool) {
        if let Some(vmt) = self.get_exportable_vmt() {
            match vmt.to_string() {
                Ok(serialized_vmt) => {
                    match std::fs::write(path, serialized_vmt) {
                        Ok(_) => {
                            // Clear backup because the modified file is now the new permanent baseline
                            self.backup_vmt_bytes = None;
                            // Update MaterialSystem with the latest version of the material
                            self.mat_sys.register(&self.current_material_name, vmt);

                            toasts::success("Material saved to disk!", 3000);
                            if should_apply {
                                push_event(OverlayEvent::EngineCommand("mat_reloadallmaterials".into()));
                            }
                        }
                        Err(e) => {
                            toasts::error(format!("File write error: {}", e), 4000);
                        }
                    }
                }
                Err(e) => {
                    toasts::error(format!("Serialization error: {:?}", e), 4000);
                }
            }
        }
    }
}

// Ensure cleanup happens if the struct is entirely dropped
impl Drop for MaterialInspector {
    fn drop(&mut self) {
        self.revert_preview();
    }
}

impl Window for MaterialInspector {
    fn name(&self) -> &'static str { "Material Inspector" }

    fn set_open(&mut self, open: bool) { self.is_open = open }
    fn is_open(&self) -> bool { self.is_open }

    fn is_should_render(&self, shared_state: &SharedState, _engine: &Engine) -> bool {
        shared_state.is_overlay_focused
    }

    fn on_event(&mut self, event: &OverlayEvent, _shared_state: &mut SharedState) {
        if matches!(event, OverlayEvent::ToggleOverlay) {
            self.revert_preview();
        }
    }

    fn draw(&mut self, ctx: &Context, _shared_state: &mut SharedState, engine: &Engine) {
        let mut open = self.is_open;

        // self.file_dialog.update(ctx);

        egui::Window::new(self.name())
            .open(&mut open)
            .resizable(true)
            .default_width(550.0)
            .default_height(600.0)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("mat_inspector_toolbar").show_inside(ui, |ui| {
                    ui.add_space(6.0);
                    self.draw_toolbar(ui, engine);
                    ui.add_space(6.0);
                });

                egui::TopBottomPanel::bottom("mat_inspector_footer").show_inside(ui, |ui| {
                    ui.add_space(6.0);
                    self.draw_footer(ui, engine);
                    ui.add_space(6.0);
                });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    if let Some(err) = &self.error_message {
                        ui.colored_label(Color32::RED, format!("⚠ Error: {}", err));
                        ui.separator();
                    }

                    // Independent variables to prevent closure borrowing issues
                    let fs = &self.mat_sys.fs;
                    let new_key = &mut self.new_prop_key;
                    let new_val = &mut self.new_prop_value;
                    let mut trigger_preview = false;
                    let mut keys_to_remove = Vec::new();
                    let mut add_prop_request = None;

                    let shift_held = ctx.input(|i| i.modifiers.shift);

                    if let Some(vmt) = &mut self.current_vmt {
                        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Shader:").strong());
                                if ui.add(TextEdit::singleline(&mut vmt.shader).desired_width(f32::INFINITY)).changed() {
                                    trigger_preview = true;
                                }
                            });

                            ui.add_space(12.0);
                            ui.label(egui::RichText::new("ℹ Hold SHIFT to delete properties")
                                .small()
                                .color(if shift_held { Color32::WHITE } else { Color32::GRAY })
                            );

                            ui.add_space(6.0);

                            if Self::draw_vmt_block(
                                ui,
                                &mut vmt.properties,
                                "root",
                                fs,
                                &mut keys_to_remove,
                                &mut self.disabled_properties,
                                shift_held
                            ) {
                                trigger_preview = true;
                            }

                            ui.add_space(24.0);

                            // A dedicated visual block for adding properties
                            egui::Frame::NONE
                                .fill(ui.visuals().faint_bg_color)
                                .inner_margin(8.0)
                                .corner_radius(4.0)
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("➕ Add New Property").strong());
                                    ui.add_space(4.0);

                                    ui.horizontal(|ui| {
                                        ui.add(TextEdit::singleline(new_key).desired_width(120.0).hint_text("Key (e.g. $color)"));

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("Add").clicked() {
                                                if !new_key.is_empty() {
                                                    add_prop_request = Some((new_key.clone(), new_val.clone()));
                                                }
                                            }
                                            ui.add(TextEdit::singleline(new_val).desired_width(f32::INFINITY).hint_text("Value"));
                                        });
                                    });
                                });
                        });

                        // Process Additions
                        if let Some((k, v)) = add_prop_request {
                            self.disabled_properties.remove(&k);
                            vmt.set_string(&k, &v);
                            trigger_preview = true;
                            new_key.clear();
                            new_val.clear();
                        }

                        // Process Deletions
                        for key in keys_to_remove {
                            vmt.remove(&key);
                            self.disabled_properties.remove(&key);
                            trigger_preview = true;
                        }

                    } else if self.error_message.is_none() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label(egui::RichText::new("No material selected.").weak().heading());
                            ui.add_space(10.0);
                            ui.label("Use the 'Pick Material' button in the toolbar to select a surface.");
                        });
                    }

                    // Apply preview if any grid text field was altered
                    if trigger_preview && self.realtime_preview {
                        self.preview_material();
                    }
                });
            });

        self.is_open = open;
    }
}
