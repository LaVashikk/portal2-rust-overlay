use crate::{SharedState, Window};
use super::{FormAction, WidgetForm};

#[derive(Debug)]
pub struct SurveyWin {
    form: WidgetForm,
    is_opened: bool,
}

impl SurveyWin {
    pub fn new(config_path: &str) -> Self {
        Self {
            form: WidgetForm::new(config_path),
            is_opened: false,
        }
    }
}

impl Window for SurveyWin {
    fn name(&self) -> &'static str { "Survey" }

    fn is_should_render(&self, _shared_state: &SharedState, engine: &engine_api::Engine) -> bool {
        let cvar_system = engine.cvar_system();
        match cvar_system.find_var("open_survey") {
            Some(val) => !val.get_string().chars().all(|c| c.is_ascii_digit()) || val.get_int() != 0,
            None => { engine.client().client_cmd("setinfo open_survey 0"); false }, // todo: init it in other place!
        }

    }

    fn draw(
        &mut self,
        ctx: &egui::Context,
        shared_state: &mut SharedState,
        engine: &engine_api::Engine,
    ) {
        // This block triggers after `load_form` or on the first opening
        if !self.is_opened {
            let target_survey_path = engine.cvar_system().find_var("open_survey")
                .map(|cvar| cvar.get_string())
                .unwrap_or_default();

            // If a new survey is requested, load it
            if !target_survey_path.chars().all(|c| c.is_ascii_digit()) && self.form.config_path != target_survey_path {
                self.form.load_form(&target_survey_path);
            }

            let client = engine.client();
            if !client.is_paused() && client.is_in_game() { client.client_cmd("pause"); }
            self.is_opened = true;
            shared_state.surver_is_opened = true;
            shared_state.is_overlay_focused = true; // todo! debug shit, needed better interface?
        }

        if self.form.draw_modal_window(ctx, engine, false) == FormAction::Submitted {
            match self.form.save_results(engine, "survey") {
                Ok(_) => {
                    let client = engine.client();
                    client.client_cmd("open_survey 0");
                    if client.is_in_game() { client.client_cmd("unpause"); }
                    self.is_opened = false;
                    shared_state.surver_is_opened = false;
                    shared_state.is_overlay_focused = false;
                    self.form.reset_state();
                }
                Err(e) => {
                    log::error!("Failed to save survey: {}", e);
                    // TODO: Display a modal window with an error?
                }
            }
        }
    }

    fn on_raw_input(&mut self, umsg: u32, _wparam: u16) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{WM_CHAR, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP};

        if !self.is_opened { return true; }
        match umsg {
            WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP | WM_CHAR => false,
            _ => true
        }
    }

    fn toggle(&mut self) { /* Controlling via CVar */ }
    fn is_open(&self) -> bool { true }
}
