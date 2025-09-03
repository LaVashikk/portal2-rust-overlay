use egui::Context;
use windows::Win32::{Foundation::{LPARAM, WPARAM}, UI::{Input::KeyboardAndMouse::VK_F3, WindowsAndMessaging::{WM_KEYUP, WM_SYSKEYDOWN}}};
use windows::Win32::UI::WindowsAndMessaging::{WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP};

use crate::{engine, overlay::utils::Window};

mod utils;

/// Shared state accessible to all windows.
#[derive(Debug, Default, Clone)]
pub struct SharedState {
    pub is_overlay_focused: bool, // todo: namening?
    pub open_counter: u64,
}

/// todo
pub struct UiManager {
    windows: Vec<Box<dyn Window + Send + Sync>>,
    input_context: Option<utils::SendableContext>,
    pub shared_state: SharedState,
    pub is_focused: bool,
}

impl UiManager {
    pub fn new() -> Self {
        Self {
            windows: vec![ // PANELS!
                Box::new(OverlayText::default()),
                Box::new(DebugWindow { is_open: true }),
                Box::new(fogui::FogWindow::default()),
                // ....
            ],
            shared_state: SharedState::default(),
            is_focused: false,
            input_context: None
        }
    }

    pub(crate) fn draw_ui(&mut self, ctx: &Context) {
        for window in self.windows.iter_mut() {
            if window.is_open() {
                // eprintln!("-- draw window: {:?}", window);
                window.draw(ctx, &mut self.shared_state);
            }
        }
    }

    /// # Arguments
    /// * `umsg`: Windows message code (e.g., WM_KEYDOWN).
    /// * `wparam`: Additional message parameter (e.g., virtual key code).
    /// * `lparam`: Additional message parameter.
    ///
    /// # Returns
    /// * `true` - if the input should be passed to the game.
    /// * `false` - if the input should be "eaten" (blocked).
    pub(crate) fn on_input(&mut self, umsg: u32, wparam: WPARAM, _lparam: LPARAM) -> bool {
        if umsg == WM_KEYUP || umsg == WM_SYSKEYDOWN {
            if wparam.0 as u16 == VK_F3.0 {
                self.toggle_focus();
                return false;
            }
        }

        // todo: if right mouse button is held down and focused - give mouse input

        if self.is_focused {
            match umsg {
                // "Eat" these messages so that the game doesn't receive them
                WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP
                | WM_MBUTTONDOWN | WM_MBUTTONUP | WM_MOUSEWHEEL => {
                    return false;
                }
                _ => {}
            }
        }

        true
    }

    ///
    pub fn toggle_focus(&mut self) {
        self.is_focused = !self.is_focused;
        self.shared_state.is_overlay_focused = self.is_focused;

        let input_stack_system = engine::get().input_stack_system();
        if self.input_context.is_none() {
            let ctx_ptr = input_stack_system.push_input_context();
            self.input_context = Some(utils::SendableContext(ctx_ptr));
        }

        let ctx = self.input_context.as_ref().unwrap().0;

        if self.is_focused {
            input_stack_system.enable_input_context(ctx, true);
            input_stack_system.set_cursor_visible(ctx, false);
            input_stack_system.set_mouse_capture(ctx, true);
            self.shared_state.open_counter += 1;
        } else {
            // TODO: Center the cursor
            // unsafe {
            //     let x = windows::Win32::UI::WindowsAndMessaging::SetCursorPos(960, 540);
            //     eprintln!("{:?}", x);
            // };
            input_stack_system.enable_input_context(ctx, false);
        }
    }
}



// ---------------------- \\
//      YOUR WINDOWS      \\
// ---------------------- \\

#[derive(Debug, Default)]
pub struct OverlayText;
impl Window for OverlayText {
    fn name(&self) -> &'static str { "Overlay Text" }
    fn toggle(&mut self) {}
    fn is_open(&self) -> bool { true }
    fn draw(&mut self, ctx: &Context, _shared_state: &mut SharedState) {
        let screen_rect = ctx.screen_rect();
        ctx.debug_painter().text(
            egui::pos2(screen_rect.left() + 10.0, screen_rect.bottom() - 10.0),
            egui::Align2::LEFT_BOTTOM,
            "IN-GAME CUSTOM OVERLAY!",
            egui::FontId::proportional(20.0),
            egui::Color32::ORANGE,
        );
    }
}


#[derive(Debug)]
pub struct DebugWindow {
    is_open: bool,
}

impl Window for DebugWindow {
    fn name(&self) -> &'static str { "Debug Window" }
    fn toggle(&mut self) { self.is_open = !self.is_open; }
    fn is_open(&self) -> bool { self.is_open }

    fn draw(&mut self, ctx: &Context, shared_state: &mut SharedState) {
        if !shared_state.is_overlay_focused {
            return
        }

        egui::Window::new(self.name())
            .open(&mut self.is_open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("CVar Inspector");
                ui.separator();

                let cvar_system = engine::get().cvar_system();
                match cvar_system.find_var("sv_cheats") {
                    Some(sv_cheats_cvar) => {
                        let value = sv_cheats_cvar.get_int();

                        ui.label(format!("sv_cheats value: {}", value));

                        if ui.button("Toggle sv_cheats").clicked() {
                           let new_value = if value == 0 { 1 } else { 0 };
                           engine::get()
                                .client()
                                .execute_client_cmd_unrestricted(&format!("sv_cheats {}", new_value));
                        }
                    }
                    None => {
                        // Impossible case, this should not happen!
                        ui.colored_label(egui::Color32::RED, "sv_cheats: <not found>");
                    }
                }
            });
    }
}

mod fogui;
