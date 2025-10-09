#![cfg(all(target_os = "windows", target_pointer_width = "32"))]
#![allow(non_snake_case)]

use std::sync::{Mutex, OnceLock};
use engine_api::Engine;
use windows::Win32::Foundation::{BOOL, HMODULE, HWND, LPARAM, WPARAM};
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F3;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR, WM_INPUT, WM_KEYUP, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDBLCLK, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDBLCLK, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_XBUTTONDBLCLK};

mod proxy;
mod hooks;
mod renderer;
mod logger;

const TEXT_SCALE: f32 = 1.4;

// The global, thread-safe instance of the entire overlay application.
// This serves as the foundation for the UI. To add new windows or views,
// implement them in the `overlay` module and register them in `UiManager::new()`.
static OVERLAY_APP: OnceLock<Mutex<UiManager>> = OnceLock::new();

/// The core controller for the UI overlay.
///
/// Owns all windows, manages global UI state (e.g., input focus),
/// and provides access to the game engine API.
pub struct UiManager {
    windows: Vec<Box<dyn overlay_ui::Window + Send>>,
    engine_instance: Engine,
    input_context: Option<SendableContext>,

    pub shared_state: overlay_ui::SharedState,
    pub is_focused: bool,
}

impl UiManager {
    pub fn new(engine_instance: Engine) -> Self {
        Self {
            windows: overlay_ui::regist_windows(),
            shared_state: overlay_ui::SharedState::default(),
            engine_instance,
            is_focused: false,
            input_context: None
        }
    }

    pub(crate) fn draw_ui(&mut self, ctx: &egui::Context) {
        for window in self.windows.iter_mut() {
            if window.is_open() && window.is_should_render(&self.shared_state, &self.engine_instance) {
                window.draw(ctx, &mut self.shared_state, &self.engine_instance);
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
    pub fn on_input(&mut self, umsg: u32, wparam: WPARAM, _lparam: LPARAM) -> bool {
        if umsg == WM_KEYUP || umsg == WM_SYSKEYDOWN {
            if wparam.0 as u16 == VK_F3.0 {
                self.toggle_focus();
                return false;
            }
        }

        // todo: if right mouse button is held down and focused - give mouse input

        let mut should_pass_to_game = true;
        for win in self.windows.iter_mut() {
            if !win.on_raw_input(umsg, wparam.0 as u16) {
                should_pass_to_game = false;
            }
        }

        if self.is_focused {
            match umsg {
                // "Eat" these messages so that the game doesn't receive them
                WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP
                | WM_LBUTTONDBLCLK | WM_RBUTTONDBLCLK | WM_MBUTTONDBLCLK | WM_XBUTTONDBLCLK
                | WM_MBUTTONDOWN | WM_MBUTTONUP | WM_MOUSEWHEEL | WM_INPUT => {
                    return false;
                }
                _ => {}
            }
        }

        should_pass_to_game
    }


    pub fn toggle_focus(&mut self) {
        self.is_focused = !self.is_focused;
        self.shared_state.is_overlay_focused = self.is_focused;

        let input_stack_system = self.engine_instance.input_stack_system();
        if self.input_context.is_none() {
            let ctx_ptr = input_stack_system.push_input_context();
            self.input_context = Some(SendableContext(ctx_ptr));
        }

        let ctx = self.input_context.as_ref().unwrap().0;

        if self.is_focused {
            input_stack_system.enable_input_context(ctx, true);
            input_stack_system.set_cursor_visible(ctx, false);
            input_stack_system.set_mouse_capture(ctx, true);
        } else {
            input_stack_system.enable_input_context(ctx, false);
        }
    }
}

// SAFETY: Used within `UiManager`, which is protected by a Mutex and lazily initialized.
//          This ensures that access to the underlying `InputContextT` is synchronized.
//
// Alternative considered: `thread_local!`, but that would be fatal if called from a rendering thread.
//          This approach is preferred because it provides synchronization, even if it introduces some overhead.
pub struct SendableContext(pub *mut engine_api::input_system::InputContextT);
unsafe impl Send for SendableContext {}

// --- INIT ---

fn initialize_systems() {
    // Initialize engine bindings
    let engine = match engine_api::Engine::initialize() {
        Ok(instance) => instance,
        Err(err) => {
            log::error!("Failed to initialize engine interfaces! Reason: {}", err);
            unsafe {
                MessageBoxA(
                    None,
                    PCSTR(b"Failed to initialize engine interfaces! The overlay will not work.\0".as_ptr()),
                    PCSTR(b"Initialization Error\0".as_ptr()),
                    MB_ICONERROR,
                );
            }
            return;
        },
    };

    // Initialize the UI application state
    if OVERLAY_APP.set(Mutex::new(
        UiManager::new(engine)
    )).is_err() {
        log::error!("App was already initialized!");
    }

    log::info!("Overlay mod initialized successfully.");
}

fn initialize_render(hwnd: HWND, device: &windows::Win32::Graphics::Direct3D9::IDirect3DDevice9) {
    renderer::initialize(hwnd, device);
    log::info!("Renderer initialized successfully.");
}

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(
    _h_inst_dll: HMODULE,
    fdw_reason: u32,
    _lpv_reserved: *mut c_void,
) -> BOOL {
    if fdw_reason == DLL_PROCESS_ATTACH {
        logger::init();
        log::info!("Logger initialized. Pre-loading D3D9 proxy functions...");

        // Pre-load the original d3d9.dll function pointers.
        proxy::initialize();
    }
    BOOL(1)
}


// --- All other proxy function exports ---
use std::ffi::c_void;
use windows::core::{HRESULT, PCSTR, PCWSTR};
use windows::Win32::Graphics::Direct3D9::{IDirect3D9, IDirect3D9Ex};

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Direct3DCreate9(sdk_version: u32) -> *mut IDirect3D9 {
    type FnType = unsafe extern "system" fn(u32) -> *mut IDirect3D9;
    let original_fn: FnType = unsafe { proxy::get_original_function(11) }; // 11 - Direct3DCreate9
    let d3d9 = unsafe { original_fn(sdk_version) };

    if !d3d9.is_null() {
        unsafe { hooks::install(d3d9) };
    }

    d3d9
}

create_proxy_function!(D3DPERF_BeginEvent, 1, (col: u32, wsz_name: PCWSTR), i32);
create_proxy_function!(D3DPERF_EndEvent, 2, (), i32);
create_proxy_function!(D3DPERF_GetStatus, 3, (), u32);
create_proxy_function!(D3DPERF_QueryRepeatFrame, 4, (), BOOL);
create_proxy_function!(D3DPERF_SetMarker, 5, (col: u32, wsz_name: PCWSTR), ());
create_proxy_function!(D3DPERF_SetOptions, 6, (dw_options: u32), ());
create_proxy_function!(D3DPERF_SetRegion, 7, (col: u32, wsz_name: PCWSTR), ());
create_proxy_function!(DebugSetLevel, 8, (level: i32), ());
create_proxy_function!(DebugSetMute, 9, (), ());
create_proxy_function!(Direct3D9EnableMaximizedWindowedModeShim, 10, (), ());
create_proxy_function!(Direct3DCreate9Ex, 12, (sdk_version: u32, out_ptr: *mut Option<IDirect3D9Ex>), HRESULT);
create_proxy_function!(Direct3DShaderValidatorCreate9, 13, (), *mut c_void);
create_proxy_function!(PSGPError, 14, (), *mut c_void);
create_proxy_function!(PSGPSampleTexture, 15, (), *mut c_void);
