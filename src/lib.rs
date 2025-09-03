#![cfg(all(target_os = "windows", target_pointer_width = "32"))]
#![allow(non_snake_case)]

use std::sync::{Mutex, OnceLock};
use windows::Win32::Foundation::{BOOL, HMODULE, HWND};
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR};

mod proxy;
mod hooks;
pub mod memory;
pub mod engine;
pub mod renderer;
pub mod overlay;
mod logger;

// --- APP ---
// The global, thread-safe instance of the entire overlay application.
// This serves as the foundation for the UI. To add new windows or views,
// implement them in the `overlay` module and register them in `UiManager::new()`.
static OVERLAY_APP: OnceLock<Mutex<overlay::UiManager>> = OnceLock::new();

fn initialize_systems() {
    eprintln!("[INIT] iNIT SOME SHIT!");

    //
    if !engine::initialize() {
        log::error!("[INIT] CRITICAL: Failed to initialize engine interfaces!");
        unsafe {
            MessageBoxA(
                None,
                PCSTR(b"Failed to initialize engine interfaces! The overlay will not work.\0".as_ptr()),
                PCSTR(b"Initialization Error\0".as_ptr()),
                MB_ICONERROR,
            );
        }
        return;
    }
    log::info!("[INIT] Engine initialized successfully.");

    //
    if OVERLAY_APP.set(Mutex::new(overlay::UiManager::new())).is_err() {
        log::info!("[ERROR] App was already initialized!");
    }

    log::info!("[INIT] Overlay mod initialized successfully.");
}


fn initialize_render(hwnd: HWND, device: &windows::Win32::Graphics::Direct3D9::IDirect3DDevice9) {
    renderer::initialize(hwnd, device);
    log::info!("[INIT] Renderer initialized successfully.");
}

#[no_mangle]
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

#[no_mangle]
pub unsafe extern "system" fn Direct3DCreate9(sdk_version: u32) -> *mut IDirect3D9 {
    type FnType = unsafe extern "system" fn(u32) -> *mut IDirect3D9;
    let original_fn: FnType = proxy::get_original_function(11); // 11 - Direct3DCreate9
    let d3d9 = original_fn(sdk_version);

    if !d3d9.is_null() {
        hooks::install(d3d9);
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
