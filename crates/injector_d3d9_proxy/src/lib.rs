//! d3d9-proxy: A proxy DLL for d3d9.dll that hooks CreateDevice.
//!
//! This crate is responsible for:
//! - Loading the original d3d9.dll and forwarding all its exports.
//! - Hooking the Direct3DCreate9 function to install a hook on the CreateDevice method of the IDirect3D9 interface.
//! - The hook is used to intercept the creation of a D3D9 device and install a present hook.
#![cfg(all(target_os = "windows", target_pointer_width = "32"))]
#![allow(non_snake_case)]

use std::ffi::c_void;
use windows::core::HRESULT;
use windows::Win32::Graphics::Direct3D9::{IDirect3D9, IDirect3D9Ex};
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::BOOL;

mod proxy;

// Pull callbacks from overlay_runtime
use overlay_runtime::CALLBACKS;

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Direct3DCreate9(sdk_version: u32) -> *mut IDirect3D9 { unsafe {
    type FnType = unsafe extern "system" fn(u32) -> *mut IDirect3D9;

    // Initialize logging and original proxy table
    overlay_runtime::logger::init();
    proxy::initialize();

    let original_fn: FnType = unsafe { proxy::get_original_function(11) }; // ord 11 - Direct3DCreate9
    let d3d9 = unsafe { original_fn(sdk_version) };

    if !d3d9.is_null() {
        // Install CreateDevice hook via shared core
        let _ = d3d9_hook_core::install_on_d3d9(d3d9, &CALLBACKS);
    }

    d3d9
}}

// Proxy other exports as before
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
