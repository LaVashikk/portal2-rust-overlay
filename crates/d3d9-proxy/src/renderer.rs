use egui_d3d9::EguiDx9;
use std::mem::transmute;
use std::sync::{Mutex, Once, OnceLock};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Direct3D9::IDirect3DDevice9;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, SetWindowLongPtrA, SetWindowLongPtrW, GWLP_WNDPROC, WNDPROC};

use crate::TEXT_SCALE;

static EGUI_RENDERER: OnceLock<Mutex<EguiDx9<()>>> = OnceLock::new();
static mut O_WNDPROC: Option<WNDPROC> = None;

unsafe extern "stdcall" fn hooked_wndproc( // todo: move it in hooks.rs
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if umsg == 533 { // lose mouse capture
        return unsafe { CallWindowProcW(O_WNDPROC.unwrap(), hwnd, umsg, wparam, lparam) };
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
        unsafe { CallWindowProcW(O_WNDPROC.unwrap(), hwnd, umsg, wparam, lparam) }
    }
}

/// Initializes Egui, the renderer, and the WndProc hook.
pub fn initialize(hwnd: HWND, device: &IDirect3DDevice9) {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        log::info!("Initializing Egui D3D9 renderer...");

        // Get access to the thread-local variable for initialization
        let egui_renderer = EguiDx9::init(
            device,
            hwnd,
            |ctx, _| _render_ui(ctx),
            (),
            false,
        );

        if EGUI_RENDERER.set(Mutex::new(egui_renderer)).is_err() {
            log::error!("Egui renderer already initialized!");
            return;
        }

        // Installing WndProc hook
        unsafe {
            O_WNDPROC = Some(transmute(SetWindowLongPtrW(
                hwnd,
                GWLP_WNDPROC,
                hooked_wndproc as usize as _,
            )));
        }

        log::info!("Egui Initialized and WndProc hooked.");
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

/// Called before D3D9 Reset to free resources.
pub fn handle_pre_reset() {
    log::warn!("D3D9 device is resetting. Releasing Egui resources.");
    if let Some(mutex) = EGUI_RENDERER.get() {
        if let Ok(mut renderer) = mutex.lock() {
            renderer.pre_reset();
        }
    }
}
/// Called after a D3D9 Reset to recreate resources.
pub fn handle_post_reset(device: &IDirect3DDevice9) {
    log::info!("D3D9 device has been reset. Recreating Egui resources...");
    if let Some(mutex) = EGUI_RENDERER.get() {
        if let Ok(mut renderer) = mutex.lock() {
            renderer.post_reset(device);
        }
    }
}

fn _render_ui(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::new(18.0*TEXT_SCALE, egui::FontFamily::Proportional)),
        (egui::TextStyle::Body, egui::FontId::new(12.5*TEXT_SCALE, egui::FontFamily::Proportional)),
        (egui::TextStyle::Monospace, egui::FontId::new(12.0*TEXT_SCALE, egui::FontFamily::Proportional)),
        (egui::TextStyle::Button, egui::FontId::new(12.5*TEXT_SCALE, egui::FontFamily::Proportional)),
        (egui::TextStyle::Small, egui::FontId::new(9.0*TEXT_SCALE, egui::FontFamily::Proportional)),
    ]
    .into();
    ctx.set_style(style);

    if let Some(app) = super::OVERLAY_APP.get() {
        if let Ok(mut app_mutex) = app.lock() {
            app_mutex.draw_ui(ctx);
        }
    }
}
