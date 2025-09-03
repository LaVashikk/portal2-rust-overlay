use egui_d3d9::EguiDx9;
use std::mem::transmute;
use std::sync::{Mutex, Once, OnceLock};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Direct3D9::IDirect3DDevice9;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, SetWindowLongPtrA, GWLP_WNDPROC, WNDPROC};

static EGUI_RENDERER: OnceLock<Mutex<EguiDx9<()>>> = OnceLock::new();
static mut O_WNDPROC: Option<WNDPROC> = None;

unsafe extern "stdcall" fn hooked_wndproc( // todo: move it in hooks.rs
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if umsg == 533 { // lose mouse capture
        return CallWindowProcW(O_WNDPROC.unwrap(), hwnd, umsg, wparam, lparam);
    }

    // pass the message to it.
    if let Some(mutex) = EGUI_RENDERER.get() {
        if let Ok(mut renderer) = mutex.lock() {
            renderer.wnd_proc(umsg, wparam, lparam);
        }
    }

    let mut should_pass_to_game = true;
    if let Some(app_mutex) = crate::OVERLAY_APP.get() {
        // For safety, use try_lock
        match app_mutex.try_lock() {
            Ok(mut app) => {
                should_pass_to_game = app.on_input(umsg, wparam, lparam);
            },
            Err(e) => {
                // debug stuff
                log::warn!("[DEADLOCK DETECTED] Mutex lock failed in hooked_wndproc: {:?}. Message: {}, wparam: {}, lparam: {}", e, umsg, wparam.0, lparam.0);
            }
        }
    }

    if !should_pass_to_game {
        LRESULT(0)
    } else {
        CallWindowProcW(O_WNDPROC.unwrap(), hwnd, umsg, wparam, lparam)
    }
}

/// Initializes Egui, the renderer, and the WndProc hook.
pub fn initialize(hwnd: HWND, device: &IDirect3DDevice9) {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        eprintln!("[MOD] Initializing Egui D3D9 renderer...");

        // Get access to the thread-local variable for initialization
        let egui_renderer = EguiDx9::init(
            device,
            hwnd,
            |ctx, _| _render_ui(ctx),
            (),
            false,
        );

        let mutex_renderer = Mutex::new(egui_renderer);

        if EGUI_RENDERER.set(mutex_renderer).is_err() {
            eprintln!("[MOD ERROR] Egui renderer already initialized!");
            return;
        }

        // Installing WndProc hook
        unsafe {
            O_WNDPROC = Some(transmute(SetWindowLongPtrA(
                hwnd,
                GWLP_WNDPROC,
                hooked_wndproc as usize as _,
            )));
        }

        eprintln!("[MOD] Egui Initialized and WndProc hooked.");
    });
}

/// Rendering function called every frame from the Present hook.
pub fn render(device: &IDirect3DDevice9) {
    if let Some(mutex) = EGUI_RENDERER.get() {
        if let Ok(mut renderer) = mutex.lock() {
            renderer.present(device);
        }
    }
}

/// Handling D3D9 device reset.
pub fn handle_device_reset() {
    eprintln!("[MOD] D3D9 device is resetting. Notifying Egui.");
    if let Some(mutex) = EGUI_RENDERER.get() {
        if let Ok(mut renderer) = mutex.lock() {
            renderer.pre_reset();
        }
    }
}

fn _render_ui(ctx: &egui::Context) {
    if let Some(app) = super::OVERLAY_APP.get() {
        if let Ok(mut app_mutex) = app.lock() {
            app_mutex.draw_ui(ctx);
        }
    }
}
