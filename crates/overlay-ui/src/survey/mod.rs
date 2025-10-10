use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use engine_api::Engine;
use crate::{SharedState, Window};

mod types;
use types::*;

#[derive(Debug, Default)]
pub struct SurveyApp {
    config: SurveyConfig,
    state: Vec<WidgetState>,
    output_path: PathBuf,
    pub opened: bool,
    current_survey_path: String,
}

fn show_error_and_panic(caption: &str, text: &str) -> ! { // utils
    use std::ffi::CString;
    use windows::core::PCSTR;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_OK, MB_ICONERROR};

    log::error!("ERROR {}: {}", caption, text);
    let lp_text = CString::new(text).unwrap();
    let lp_caption = CString::new(caption).unwrap();
    unsafe {
        MessageBoxA(
            None,
            PCSTR(lp_text.as_ptr() as *const u8),
            PCSTR(lp_caption.as_ptr() as *const u8),
            MB_OK | MB_ICONERROR,
        );
    }
    panic!("{}", text);
}

impl SurveyApp {
    pub fn new(config: &str) -> Self {
        let mut app = Self::default();
        app.load_survey(config);
        app
    }

    /// Loads and initializes a survey from a configuration file.
    /// This method resets all previous states.
    fn load_survey(&mut self, config_path_str: &str) {
        let config_path = PathBuf::from(config_path_str);

        // Read the configuration file into a string
        let json_str = match fs::read_to_string(&config_path) {
            Ok(s) => s,
            Err(e) => {
                let error_text = format!(
                    "Failed to read configuration file '{}':\n\n{}",
                    config_path.display(),
                    e
                );
                show_error_and_panic("File Read Error", &error_text); // TODO: WHAT THE FUCK LOL NO AHAHAHHA
            }
        };

        // Parse the JSON string into the SurveyConfig struct
        let config: SurveyConfig = match serde_json::from_str(&json_str) {
            Ok(c) => c,
            Err(e) => {
                let error_text = format!(
                    "Failed to parse JSON from file '{}':\n\n{}",
                    config_path.display(),
                    e
                );
                show_error_and_panic("Configuration Error", &error_text);
            }
        };

        // Initialize the state based on the loaded config
        let state = config
            .widgets
            .iter()
            .map(|w_config| match w_config {
                WidgetConfig::OneToTen(_) => WidgetState::OneToTen(None),
                WidgetConfig::Essay(_) => WidgetState::Essay(String::new()),
                WidgetConfig::RadioChoices(_) => WidgetState::RadioChoices(None),
            })
            .collect();

        // Dynamically determine the path for saving results
        let mut output_path = config_path.clone();
        let stem = output_path
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        output_path.set_file_name(format!("{}_result.json", stem));

        // Update the object's state
        self.config = config;
        self.state = state;
        self.output_path = output_path;
        self.current_survey_path = config_path_str.to_string();
        self.opened = false; // Reset the flag so that the `if !self.opened` trigger works
    }

    fn are_all_required_filled(&self) -> bool {
        self.config
            .widgets
            .iter()
            .zip(self.state.iter())
            .all(|(config, state)| !config.is_required() || state.is_answered())
    }

    /// Collects all data and saves it to a structured JSON file.
    fn save_answer(&self, engine: &Engine) -> Result<(), String> {
        // 1. Collect metadata
        let client = engine.client();
        let local_player_idx = client.get_local_player();
        let (user_name, user_xuid) = client
            .get_player_info(local_player_idx)
            .map(|info| (info.name().to_string(), info.xuid.to_string()))
            .unwrap_or_else(|| ("<unknown>".to_string(), "0".to_string()));

        let survey_id = self.current_survey_path.clone();

        // 2. Format answers as "question: answer"
        let mut answers = HashMap::new();
        for (config, state) in self.config.widgets.iter().zip(self.state.iter()) {
            answers.insert(config.text().to_string(), state.clone());
        }

        // 3. Create the final structure
        let submission = SurveySubmission {
            survey_id,
            user_name,
            user_xuid,
            map_name: client.get_level_name().to_string(),
            timestamp: client.get_last_time_stamp(),
            answers,
        };

        // 4. Serialize and save
        let json_data = serde_json::to_string_pretty(&submission).map_err(|e| e.to_string())?;
        fs::write(&self.output_path, json_data).map_err(|e| e.to_string())?;

        Ok(())
    }

    fn reset_state(&mut self) {
        self.state = self.config
            .widgets
            .iter()
            .map(|w_config| match w_config {
                WidgetConfig::OneToTen(_) => WidgetState::OneToTen(None),
                WidgetConfig::Essay(_) => WidgetState::Essay(String::new()),
                WidgetConfig::RadioChoices(_) => WidgetState::RadioChoices(None),
            })
            .collect();
    }

    fn render_widgets(&mut self, ui: &mut egui::Ui) {
        for (widget_config, widget_state) in self.config.widgets.iter().zip(self.state.iter_mut())
        {
            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(15, 0))
                .show(ui, |ui| {
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        ui.horizontal_wrapped(|ui| {
                            let heading = egui::RichText::new(widget_config.text()).strong();
                            ui.label(heading);
                            if widget_config.is_required() && !widget_state.is_answered() {
                                ui.colored_label(egui::Color32::RED, " *");
                            }
                        });
                    });
                    ui.add_space(5.0);
                    match (widget_config, widget_state) {
                        (WidgetConfig::OneToTen(config), WidgetState::OneToTen(value)) => {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&config.label_at_one);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(&config.label_at_ten);
                                        },
                                    );
                                });
                                ui.add_space(5.0);
                                ui.columns(10, |columns| {
                                    for i in 0..10 {
                                        let num = (i + 1) as u8;
                                        columns[i].vertical_centered(|ui| {
                                            ui.selectable_value(
                                                value,
                                                Some(num),
                                                (i + 1).to_string(),
                                            );
                                        });
                                    }
                                });
                            });
                        }
                        (WidgetConfig::Essay(_config), WidgetState::Essay(text)) => {
                            ui.add(
                                egui::TextEdit::multiline(text)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(5),
                            );
                        }
                        (WidgetConfig::RadioChoices(config), WidgetState::RadioChoices(selected)) => {
                            ui.vertical(|ui| {
                                for choice in &config.choices {
                                    ui.radio_value(selected, Some(choice.clone()), choice);
                                }
                            });
                        }
                        _ => {
                            unreachable!("Типы виджета в конфиге и состоянии не совпадают.");
                        }
                    }
                });
            ui.separator();
        }
    }
}

impl Window for SurveyApp {
    fn is_should_render(&self, _shared_state: &SharedState, engine: &engine_api::Engine) -> bool {
        let cvar_system = engine.cvar_system();
        match cvar_system.find_var("open_survey") { // STUPID LMAO AHAHAHAH
            Some(val) => !val.get_string().chars().all(|c| c.is_ascii_digit()) || val.get_int() != 0,
            None => {
                engine.client().client_cmd("setinfo open_survey 0");
                false
            }
        }
    }

    fn draw(
        &mut self,
        ctx: &egui::Context,
        shared_state: &mut SharedState,
        engine: &engine_api::Engine,
    ) {
        // This block triggers after `load_survey` or on the first opening
        if !self.opened {
            let target_survey_path = engine.cvar_system().find_var("open_survey")
                .map(|cvar| cvar.get_string())
                .unwrap_or_default();

            // If a new survey is requested, load it
            if !target_survey_path.chars().all(|c| c.is_ascii_digit()) && self.current_survey_path != target_survey_path {
                self.load_survey(&target_survey_path);
            }

            let client = engine.client();
            if !client.is_paused() && client.is_in_game() { client.client_cmd("pause"); }
            self.opened = true;
            shared_state.is_overlay_focused = true; // todo! debug shit, needed better interface?
        }

        let modal_id = egui::Id::new("survey_modal_id");
        let area = egui::Modal::default_area(modal_id)
            .default_size(ctx.screen_rect().size() * 0.6);

        let modal = egui::Modal::new(modal_id)
            .frame(egui::Frame::NONE)
            .area(area);

        modal.show(ctx, |ui| {
            egui::Frame::window(ui.style()).show(ui, |ui| {
                let min_scroll = ctx.screen_rect().size().y * 0.7;
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.heading(&self.config.title);
                });
                ui.add_space(10.0);
                ui.separator();

                egui::ScrollArea::vertical().min_scrolled_height(min_scroll).show(ui, |ui| {
                    self.render_widgets(ui);
                    ui.add_space(20.0);

                    ui.vertical_centered(|ui| {
                        let all_required_filled = self.are_all_required_filled();
                        let submit_button = egui::Button::new("Submit").min_size(egui::vec2(120.0, 30.0));

                        if ui.add_enabled(all_required_filled, submit_button).clicked() {
                            match self.save_answer(engine) {
                                Ok(_) => { // stupid toggle todo dev shit frfr
                                    let client = engine.client();
                                    client.client_cmd("open_survey 0");
                                    if client.is_in_game() { client.client_cmd("unpause"); }
                                    self.opened = false;
                                    shared_state.is_overlay_focused = false;
                                    self.reset_state();
                                }
                                Err(e) => {
                                    log::error!("Failed to save survey: {}", e);
                                    // TODO: Show a modal window with the error
                                }
                            }
                        }
                    });
                });
            });
        });
    }

    fn on_raw_input(&mut self, umsg: u32, _wparam: u16) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{WM_CHAR, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP};

        if !self.opened { return true; }
        match umsg {
            WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP | WM_CHAR => false,
            _ => true
        }
    }

    fn name(&self) -> &'static str {
        "Survey"
    }

    fn toggle(&mut self) {
        // Opening/closing is now controlled via the "open_survey" CVar
    }

    fn is_open(&self) -> bool {
        true
    }
}
