//! client-wrapper: Wraps client_original.dll and initializes the overlay via offsets.
//!
//! - Loads client_original.dll from the same folder (absolute path).
//! - Forwards CreateInterface to the original DLL.
//! - Starts D3D9 hook via offsets in a background thread.

#![cfg(all(target_os = "windows", target_pointer_width = "32"))]

use std::ffi::{c_char, c_void};
use std::path::PathBuf;
use std::os::windows::ffi::{OsStringExt, OsStrExt};
use std::ptr::null_mut;
use std::sync::Once;

use windows::{
    core::{PCWSTR, PCSTR},
    Win32::{
        Foundation::{BOOL, HMODULE, TRUE, FALSE},
        System::LibraryLoader::{GetModuleFileNameW, LoadLibraryW, GetProcAddress},
        UI::WindowsAndMessaging::{MessageBoxW, MB_OK},
    },
};

static mut ORIGINAL_DLL: HMODULE = HMODULE(null_mut());
static INIT_ONCE: Once = Once::new();

type CreateInterfaceFn = unsafe extern "C" fn(name: *const c_char, return_code: *mut i32) -> *mut c_void;

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "system" fn DllMain(
    hinstDLL: HMODULE,
    fdwReason: u32,
    _lpvReserved: *mut c_void,
) -> BOOL {
    overlay_runtime::logger::init();
    use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
    if fdwReason == DLL_PROCESS_ATTACH {
        unsafe {
            // Resolve absolute path to client_original.dll sitting next to us.
            let mut path_buf: [u16; 260] = [0; 260];
            GetModuleFileNameW(Some(hinstDLL), &mut path_buf);
            let len = path_buf.iter().position(|&c| c == 0).unwrap_or(260);
            let path_os_string = std::ffi::OsString::from_wide(&path_buf[..len]);
            let mut dll_path = PathBuf::from(path_os_string);

            if !dll_path.pop() {
                log::error!("Failed to get DLL folder");
                let text: Vec<u16> = "Failed to get DLL folder!\0".encode_utf16().collect();
                let caption: Vec<u16> = "Proxy Error\0".encode_utf16().collect();
                MessageBoxW(None, PCWSTR(text.as_ptr()), PCWSTR(caption.as_ptr()), MB_OK);
                return FALSE;
            }

            dll_path.push("client_original.dll");

            let full_path_wide: Vec<u16> = dll_path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

            match LoadLibraryW(PCWSTR(full_path_wide.as_ptr())) {
                Ok(handle) => {
                    ORIGINAL_DLL = handle;
                }
                Err(e) => {
                    let err_msg = format!("Failed to load client_original.dll!\nCode: {}\0", e.code().0);
                    log::error!("{}", err_msg);
                    let wide_err_msg: Vec<u16> = err_msg.encode_utf16().chain(std::iter::once(0)).collect();
                    let caption: Vec<u16> = "Proxy Error\0".encode_utf16().collect();
                    MessageBoxW(None, PCWSTR(wide_err_msg.as_ptr()), PCWSTR(caption.as_ptr()), MB_OK);
                    return FALSE;
                }
            }
        }
    }
    TRUE
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C" fn CreateInterface(name: *const c_char, return_code: *mut i32) -> *mut c_void {

    // Start offsets-based D3D9 hooking exactly as before: delayed thread + offsets.
    INIT_ONCE.call_once(|| {
        // Using the shared core + overlay callbacks
        d3d9_hook_core::start_offsets_hook_thread(
            &[0xDA5D8usize, 0x179F38usize],
            5000, // 5 seconds delay same as before
            &overlay_runtime::CALLBACKS,
        );
    });

    // Forward CreateInterface to the original client DLL
    const PROC_NAME: &[u8] = b"CreateInterface\0";
    let original_create_interface = unsafe { GetProcAddress(ORIGINAL_DLL, PCSTR(PROC_NAME.as_ptr())) };
    if original_create_interface.is_none() {
        unsafe {
            let text: Vec<u16> = "CreateInterface not found in client_original.dll!\0".encode_utf16().collect();
            let caption: Vec<u16> = "Proxy Error\0".encode_utf16().collect();
            MessageBoxW(None, PCWSTR(text.as_ptr()), PCWSTR(caption.as_ptr()), MB_OK);
        }
        return null_mut();
    }

    let original_fn: CreateInterfaceFn = unsafe { std::mem::transmute(original_create_interface) };
    unsafe { original_fn(name, return_code) }
}
