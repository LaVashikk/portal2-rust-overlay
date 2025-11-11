//! overlay_runtime: Owns UiManager, global state, WndProc hook, and egui backend glue.
//!
//! Requirements respected:
//! - UiManager is a global singleton (OnceLock<Mutex<...>>).
//! - WndProc hook is NOT inside egui_backend; it lives here and cooperates with UiManager.
//! - This crate provides callbacks for d3d9_hook_core.

#![cfg(all(target_os = "windows", target_pointer_width = "32"))]

use std::sync::{Mutex, Once, OnceLock};
use windows::core::PCSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, WNDPROC, MessageBoxA, MB_ICONERROR,
    WM_CHAR, WM_INPUT, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MBUTTONDBLCLK, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_RBUTTONDBLCLK, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDBLCLK,
};
use windows::Win32::Graphics::Direct3D9::IDirect3DDevice9;

pub use d3d9_hook_core::Callbacks;
pub mod logger;

const TEXT_SCALE: f32 = 1.25;

pub static OVERLAY_RUNTIME: OnceLock<Mutex<UiManager>> = OnceLock::new();
static EGUI_RENDERER: OnceLock<Mutex<egui_backend::EguiDx9Lite>> = OnceLock::new();
static mut O_WNDPROC: Option<WNDPROC> = None;
// Store the window handle used for WndProc hook restoration on unload.
#[derive(Clone, Copy)]
struct SyncHWND(HWND);
unsafe impl Send for SyncHWND {}
unsafe impl Sync for SyncHWND {}
static FOCUS_HWND: OnceLock<SyncHWND> = OnceLock::new();

/// Overlay controller that owns custom_windows windows and talks to source-sdk.
pub struct UiManager {
    windows: Vec<Box<dyn custom_windows::Window + Send>>,
    engine_instance: source_sdk::Engine,
    input_context: Option<SendableContext>,
    cursor_visible_in_gui: bool,
    egui_wants_keyboard: bool,
    egui_wants_pointer: bool,

    pub shared_state: custom_windows::SharedState,
    pub is_focused: bool,
}

pub struct SendableContext(pub *mut source_sdk::input_system::InputContextT);
unsafe impl Send for SendableContext {}

impl UiManager {
    pub fn new(engine_instance: source_sdk::Engine) -> Self {
        Self {
            windows: custom_windows::regist_windows(),
            shared_state: custom_windows::SharedState::default(),
            engine_instance,
            is_focused: false,
            cursor_visible_in_gui: false,
            input_context: None,
            egui_wants_keyboard: false,
            egui_wants_pointer: false,
        }
    }

    pub(crate) fn draw_ui(&mut self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::new(18.0*TEXT_SCALE, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(12.5*TEXT_SCALE, egui::FontFamily::Proportional)),
            (egui::TextStyle::Monospace, egui::FontId::new(12.0*TEXT_SCALE, egui::FontFamily::Proportional)),
            (egui::TextStyle::Button, egui::FontId::new(12.5*TEXT_SCALE, egui::FontFamily::Proportional)),
            (egui::TextStyle::Small, egui::FontId::new(9.0*TEXT_SCALE, egui::FontFamily::Proportional)),
        ].into();
        ctx.set_style(style);

        for window in self.windows.iter_mut() {
            if window.is_open() && window.is_should_render(&self.shared_state, &self.engine_instance) {
                window.draw(ctx, &mut self.shared_state, &self.engine_instance);
            }
        }

        self.egui_wants_keyboard = ctx.wants_keyboard_input();
        self.egui_wants_pointer = ctx.wants_pointer_input();
    }

    /// Raw input routing. Returns true to pass input to the game, false to consume.
    pub fn on_input(&mut self, umsg: u32, wparam: WPARAM, _lparam: LPARAM) -> bool {
        use windows::Win32::UI::Input::KeyboardAndMouse::VK_F3;

        if matches!(umsg, WM_KEYUP | WM_SYSKEYDOWN) && wparam.0 as u16 == VK_F3.0 {
            self.is_focused = !self.is_focused;
            self.shared_state.is_overlay_focused = self.is_focused;
        }

        let ui_demands_cursor = self.is_focused || self.egui_wants_pointer;
        if ui_demands_cursor != self.cursor_visible_in_gui {
            let input_stack_system = self.engine_instance.input_stack_system();

            if self.input_context.is_none() {
                let ctx_ptr = input_stack_system.push_input_context();
                self.input_context = Some(SendableContext(ctx_ptr));
            }
            let ctx = self.input_context.as_ref().unwrap().0;

            if ui_demands_cursor {
                log::debug!("GRAD INPUT FROM GAME!");
                input_stack_system.enable_input_context(ctx, true);
                input_stack_system.set_cursor_visible(ctx, false);
                input_stack_system.set_mouse_capture(ctx, true);
            } else {
                log::debug!("Releasing mouse to game.");
                input_stack_system.enable_input_context(ctx, false);
            }
            self.cursor_visible_in_gui = ui_demands_cursor;
        }

        let mut should_pass_to_game = true;
        for win in self.windows.iter_mut() {
            if !win.on_raw_input(umsg, wparam.0 as u16) {
                log::debug!("Raw input handled by window '{}'. Not passing to game", win.name());
                should_pass_to_game = false;
            }
        }

        if !should_pass_to_game {
            return false;
        }

        let is_mouse_msg = matches!(
            umsg,
            WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP
            | WM_LBUTTONDBLCLK | WM_RBUTTONDBLCLK | WM_MBUTTONDBLCLK | WM_XBUTTONDBLCLK
            | WM_MBUTTONDOWN | WM_MBUTTONUP | WM_MOUSEWHEEL | WM_INPUT
        );
        if is_mouse_msg && (self.is_focused || self.egui_wants_pointer) {
            return false;
        }

        let is_keyboard_msg = matches!(umsg, WM_KEYUP | WM_KEYDOWN | WM_SYSKEYDOWN | WM_SYSKEYUP | WM_CHAR);
        if is_keyboard_msg && ((self.egui_wants_keyboard && self.egui_wants_pointer) || self.is_focused) {
            return false;
        }

        true
    }
}

fn initialize_engine_and_app() {
    match source_sdk::Engine::initialize() {
        Ok(instance) => {
            if OVERLAY_RUNTIME.set(Mutex::new(UiManager::new(instance))).is_err() {
                log::error!("UiManager was already initialized!");
            }
        }
        Err(err) => {
            log::error!("Failed to initialize engine interfaces: {}", err);
            unsafe {
                MessageBoxA(
                    None,
                    PCSTR(b"Failed to initialize engine interfaces! The overlay will not work.\0".as_ptr()),
                    PCSTR(b"Initialization Error\0".as_ptr()),
                    MB_ICONERROR,
                );
            }
        }
    }
}

/// Called by d3d9_hook_core on first valid device.
pub fn on_device_created(hwnd: HWND, device: &IDirect3DDevice9) {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = FOCUS_HWND.set(SyncHWND(hwnd));

        initialize_engine_and_app();

        let renderer = egui_backend::EguiDx9Lite::init(device, hwnd, false);
        if EGUI_RENDERER.set(Mutex::new(renderer)).is_err() {
            log::warn!("Egui renderer already initialized!");
        }

        unsafe {
            O_WNDPROC = Some(std::mem::transmute(SetWindowLongPtrW(
                hwnd,
                GWLP_WNDPROC,
                hooked_wndproc as usize as i32,
            )));
        }
    });
}

/// Called by d3d9_hook_core every Present.
pub fn on_present(device: &IDirect3DDevice9) {
    if let Some(mutex) = EGUI_RENDERER.get() {
        if let Ok(mut renderer) = mutex.lock() {
            renderer.present(device, |ctx| {
                if let Some(app) = OVERLAY_RUNTIME.get() {
                    if let Ok(mut app) = app.lock() {
                        app.draw_ui(ctx);
                    }
                }
            });
        }
    }
}

/// Called by d3d9_hook_core before Reset.
pub fn on_pre_reset() {
    if let Some(mutex) = EGUI_RENDERER.get() {
        if let Ok(mut renderer) = mutex.lock() {
            renderer.pre_reset();
        }
    }
}

/// Called by d3d9_hook_core after Reset.
pub fn on_post_reset(device: &IDirect3DDevice9) {
    if let Some(mutex) = EGUI_RENDERER.get() {
        if let Ok(mut renderer) = mutex.lock() {
            renderer.post_reset(device);
        }
    }
}

/// Our global WndProc hook. Collects input for egui backend and consults UiManager about routing.
pub unsafe extern "stdcall" fn hooked_wndproc(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT { unsafe {
    // Preserve special-case: lose mouse capture messages
    if umsg == 533 {
        return CallWindowProcW(O_WNDPROC.unwrap(), hwnd, umsg, wparam, lparam);
    }

    // Feed input into egui backend
    if let Some(mutex) = EGUI_RENDERER.get() {
        if let Ok(mut renderer) = mutex.lock() {
            renderer.wnd_proc(umsg, wparam, lparam);
        }
    }

    // Route via UiManager
    let mut should_pass_to_game = true;
    if let Some(app_mutex) = OVERLAY_RUNTIME.get() {
        if let Ok(mut app) = app_mutex.try_lock() {
            should_pass_to_game = app.on_input(umsg, wparam, lparam);
        }
    }

    if !should_pass_to_game {
        LRESULT(0)
    } else {
        CallWindowProcW(O_WNDPROC.unwrap(), hwnd, umsg, wparam, lparam)
    }
}}

pub static CALLBACKS: Callbacks = Callbacks {
    on_device_created,
    on_pre_reset,
    on_post_reset,
    on_present,
};

/// Restores original WndProc and clears overlay state.
/// Safe to call multiple times; no-op if nothing to restore.
pub fn uninstall_overlay() {
    #![allow(static_mut_refs)]
    unsafe {
        if let Some(SyncHWND(hwnd)) = FOCUS_HWND.get().copied() {
            if let Some(old) = O_WNDPROC.take() {
                // Restore original WndProc
                let _ = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, old.unwrap() as usize as i32);
            }
        }
    }
    // EGUI_RENDERER and overlay_runtime will be dropped with DLL, but we detach hooks now.
}
