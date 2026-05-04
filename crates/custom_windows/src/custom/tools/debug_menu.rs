use egui::{Context, Grid, ScrollArea, Slider, Ui, Vec2};
use portal2_sdk::Engine;
use portal2_sdk::types::{Ray_t, TraceFilter, MaskFlags};

use crate::{SharedState, Window};

#[derive(Default, PartialEq)]
enum DebugTab {
    #[default]
    Performance,
    MatSys,
    Renderer,
    Entities,
}

#[derive(Default)]
pub struct DebugMenu {
    is_open: bool,
    current_tab: DebugTab,
}

impl Window for DebugMenu {
    fn name(&self) -> &'static str { "Debug Menu" }

    fn set_open(&mut self, open: bool) { self.is_open = open; }
    fn is_open(&self) -> bool { self.is_open }

    fn is_should_render(&self, shared_state: &SharedState, _engine: &Engine) -> bool {
        shared_state.is_overlay_focused
    }

    fn draw(&mut self, ctx: &Context, _shared_state: &mut SharedState, engine: &Engine) {
        let mut open = self.is_open;

        egui::Window::new(self.name())
            .open(&mut open)
            .resizable(true)
            .default_width(500.0)
            .default_height(350.0)
            .min_size(Vec2::new(450.0, 250.0))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);

                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.current_tab, DebugTab::Performance, "Performance");
                    ui.selectable_value(&mut self.current_tab, DebugTab::MatSys, "MatSys");
                    ui.selectable_value(&mut self.current_tab, DebugTab::Renderer, "Renderer");
                    ui.selectable_value(&mut self.current_tab, DebugTab::Entities, "Entities");
                });

                ui.separator();

                ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    match self.current_tab {
                        DebugTab::Performance => self.render_performance(ui, engine),
                        DebugTab::MatSys => self.render_matsys(ui, engine),
                        DebugTab::Renderer => self.render_renderer(ui, engine),
                        DebugTab::Entities => self.render_entities(ui, engine),
                    }
                });
            });

        self.is_open = open;
    }
}

impl DebugMenu {
    fn render_performance(&self, ui: &mut Ui, engine: &Engine) {
        draw_cvar_checkbox(ui, engine, "host_speeds", "host_speeds");
        draw_cvar_checkbox(ui, engine, "cl_showfps", "Show FPS");
        draw_cvar_checkbox(ui, engine, "cl_showpos", "Show Position");
    }

    fn render_matsys(&self, ui: &mut Ui, engine: &Engine) {
        // Group sliders at the top for better visual hierarchy
        draw_cvar_slider_int(ui, engine, "mat_fullbright", "mat_fullbright", 0..=2);
        ui.add_space(4.0);
        draw_cvar_slider_int(ui, engine, "mat_debug_postprocessing_effects", "mat_debug_postprocessing_effects", 0..=2);

        ui.add_space(16.0);

        // Group checkboxes in a 2-column grid to fix the empty space issue
        let left_col =[
            "mat_bufferprimitives",
            "mat_luxels",
            "mat_leafvis",
            "mat_reversedepth",
            "mat_show_histogram",
        ];

        let right_col =[
            "mat_wireframe",
            "mat_drawflat",
            "mat_normals",
            "mat_postprocess_enable",
            "", // Empty slot to balance the 5x2 grid
        ];

        Grid::new("matsys_grid")
            .num_columns(2)
            .spacing(Vec2::new(64.0, 8.0)) // Match the Renderer tab styling
            .show(ui, |ui| {
                for i in 0..5 {
                    if !left_col[i].is_empty() {
                        draw_cvar_checkbox(ui, engine, left_col[i], left_col[i]);
                    } else {
                        ui.label("");
                    }

                    if !right_col[i].is_empty() {
                        draw_cvar_checkbox(ui, engine, right_col[i], right_col[i]);
                    } else {
                        ui.label("");
                    }
                    ui.end_row();
                }
            });
    }

    fn render_renderer(&self, ui: &mut Ui, engine: &Engine) {
        let left_col =[
            ("r_drawskybox", "Draw Skybox"),
            ("r_drawvgui", "Draw VGUI"),
            ("r_drawworld", "Draw World"),
            ("r_drawentities", "Draw Entities"),
            ("r_drawviewmodel", "Draw Viewmodel"),
            ("r_drawstaticprops", "Draw Static Props"),
            ("r_drawdetailprops", "Draw Detail Props"),
            ("r_drawlightinfo", "Draw Light Info"),
        ];

        let right_col =[
            ("r_drawdecals", "Draw Decals"),
            ("r_drawbrushmodels", "Draw Brush Models"),
            ("r_drawclipbrushes", "Draw Clip Brushes"),
            ("r_drawsprites", "Draw Sprites"),
            ("r_drawparticles", "Draw Particles"),
            ("r_drawrain", "Draw Rain"),
            ("vcollide_wireframe", "Draw VPhysics Objects"),
            ("", ""),
        ];

        Grid::new("renderer_grid")
            .num_columns(2)
            .spacing(Vec2::new(64.0, 8.0))
            .show(ui, |ui| {
                for i in 0..8 {
                    if !left_col[i].0.is_empty() {
                        draw_cvar_checkbox(ui, engine, left_col[i].0, left_col[i].1);
                    } else {
                        ui.label("");
                    }

                    if !right_col[i].0.is_empty() {
                        draw_cvar_checkbox(ui, engine, right_col[i].0, right_col[i].1);
                    } else {
                        ui.label("");
                    }
                    ui.end_row();
                }
            });
    }

    fn render_entities(&self, ui: &mut Ui, engine: &Engine) {
        let trace_result = get_entity_under_crosshair(engine);

        // Render the smart target info box
        egui::Frame::window(ui.style())
            .fill(egui::Color32::from_black_alpha(100))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Target:").strong());
                    if let Some((classname, dist)) = &trace_result {
                        if classname == "world" || classname == "worldspawn" || classname.is_empty() {
                            ui.label(egui::RichText::new("World Geometry").color(egui::Color32::GRAY));
                        } else {
                            ui.label(egui::RichText::new(classname).color(egui::Color32::GREEN));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(format!("{:.0} units", dist)).weak());
                            });
                        }
                    } else {
                        ui.label(egui::RichText::new("None").color(egui::Color32::DARK_GRAY));
                    }
                });
            });

        ui.add_space(12.0);
        ui.label("Point your crosshair at the entity you wish to affect before pressing these buttons!");
        ui.add_space(12.0);

        let is_valid_entity = match &trace_result {
            Some((classname, _)) => classname != "worldspawn" && !classname.is_empty(),
            None => false,
        };

        let buttons =[
            ("Show Debug Information", "ent_text"),
            ("Show Name", "ent_name"),
            ("Show Bounding Box", "ent_bbox"),
            ("Show Messages", "ent_messages"),
            ("Pivot", "ent_pivot"),
        ];

        ui.add_enabled_ui(is_valid_entity, |ui| {
            for (label, cmd) in buttons {
                if ui.add_sized([ui.available_width(), 24.0], egui::Button::new(label)).on_hover_text(cmd).clicked() {
                    execute(engine, cmd);
                }
                ui.add_space(4.0);
            }
        });
    }
}

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

fn draw_cvar_slider_int(ui: &mut Ui, engine: &Engine, cvar_name: &str, label: &str, range: std::ops::RangeInclusive<i32>) {
    if let Some(cvar) = engine.cvar_system().find_var(cvar_name) {
        let mut value = cvar.get_int();

        ui.horizontal(|ui| {
            // Increased width from 140.0 to 220.0 to prevent overlapping with long cvar names
            // like "mat_debug_postprocessing_effects"
            ui.allocate_ui(Vec2::new(220.0, 0.0), |ui| {
                ui.label(label).on_hover_text(cvar_name);
            });

            let slider = Slider::new(&mut value, range)
                .show_value(true)
                .trailing_fill(true);

            if ui.add_sized([ui.available_width(), 20.0], slider).changed() {
                execute(engine, &format!("{} {}", cvar_name, value));
            }
        });
    } else {
        ui.horizontal(|ui| {
            ui.allocate_ui(Vec2::new(220.0, 0.0), |ui| {
                ui.add_enabled(false, egui::Label::new(label))
                    .on_hover_text(format!("ConVar '{}' not found", cvar_name));
            });
            ui.add_enabled(false, Slider::new(&mut 0, range).show_value(true));
        });
    }
}

fn get_entity_under_crosshair(engine: &Engine) -> Option<(String, f32)> {
    let e = engine.entities();
    let local_player = e.find_by_classname(None, "player");

    let server_tools = engine.server_tools();
    if let Some((pos, angles)) = server_tools.get_player_position(None) {
        let eye_pos = pos;
        let forward = angles.to_forward_vector();
        let end_pos = eye_pos + (forward * 8192.0);

        let ray = Ray_t::new(eye_pos, end_pos);
        let mut filter = TraceFilter::new(local_player.as_deref());

        let trace = engine.engine_trace().trace_ray(&ray, MaskFlags::SOLID, &mut filter);

        if trace.fraction < 1.0 {
            let hit_ent_name = trace.hit_entity()
                .map(|ent| ent.get_classname())
                .unwrap_or_else(|| "world".to_string());

            let dist = trace.fraction * 8192.0;
            return Some((hit_ent_name, dist));
        }
    }

    None
}
