use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::sync::LazyLock;
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::FARPROC;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::System::SystemInformation::GetSystemDirectoryW;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR};

pub struct D3d9Functions {
    functions: [FARPROC; 15],
}

// pub static mut GAME_HWND: Option<HWND> = None;
pub static D3D9_PROXY: LazyLock<D3d9Functions> = LazyLock::new(|| {
    // Construct the path to the original d3d9.dll in the System32 folder.
    let mut system_path_buf = vec![0u16; 260];
    let len = unsafe { GetSystemDirectoryW(Some(&mut system_path_buf)) } as usize;
    system_path_buf.truncate(len);
    let mut path = PathBuf::from(std::ffi::OsString::from_wide(&system_path_buf));
    path.push("d3d9.dll");

    // Load the original library.
    let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let lib_handle = match unsafe { LoadLibraryW(PCWSTR(wide_path.as_ptr())) } {
        Ok(h) => h,
        Err(_) => {
            log::error!("Cannot load original d3d9.dll library");
            unsafe {
                MessageBoxA(
                    None,
                    PCSTR(b"Cannot load original d3d9.dll library.\0".as_ptr()),
                    PCSTR(b"Proxy Error\0".as_ptr()),
                    MB_ICONERROR,
                );
            }
            std::process::exit(0);
        }
    };

    // An array of all function names to be proxied.
    const NAMES: [&[u8]; 15] = [
        b"D3DPERF_BeginEvent\0", b"D3DPERF_EndEvent\0", b"D3DPERF_GetStatus\0",
        b"D3DPERF_QueryRepeatFrame\0", b"D3DPERF_SetMarker\0", b"D3DPERF_SetOptions\0",
        b"D3DPERF_SetRegion\0", b"DebugSetLevel\0", b"DebugSetMute\0",
        b"Direct3D9EnableMaximizedWindowedModeShim\0", b"Direct3DCreate9\0",
        b"Direct3DCreate9Ex\0", b"Direct3DShaderValidatorCreate9\0", b"PSGPError\0",
        b"PSGPSampleTexture\0",
    ];

    // Retrieve and store the address of each function.
    let mut functions: [FARPROC; 15] = [None; 15];
    for (i, &name) in NAMES.iter().enumerate() {
        let proc_name = PCSTR::from_raw(name.as_ptr());
        if let Some(addr) = unsafe { GetProcAddress(lib_handle, proc_name) } {
            functions[i] = Some(addr);
        } else {
            let error_message = format!(
                "Cannot find function {:?} in original d3d9.dll\0",
                String::from_utf8_lossy(name)
            );
            log::error!("Proxy error! {}", error_message);
            unsafe {
                MessageBoxA(
                    None,
                    PCSTR(error_message.as_ptr()),
                    PCSTR(b"Proxy Error\0".as_ptr()),
                    MB_ICONERROR,
                );
            }
            std::process::exit(0);
        }
    }
    D3d9Functions { functions }
});

pub fn initialize() {
    LazyLock::force(&D3D9_PROXY);
}

#[inline]
pub unsafe fn get_original_function<T>(ordinal: usize) -> T {
    let func_ptr = D3D9_PROXY.functions[ordinal - 1].unwrap();
    unsafe { std::mem::transmute_copy(&func_ptr) }
}

// ======================================================== \\
#[macro_export]
macro_rules! create_proxy_function {
    ($fn_name:ident, $ordinal:expr, ($($arg_name:ident: $arg_type:ty),*), $ret_type:ty) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "system" fn $fn_name($($arg_name: $arg_type),*) -> $ret_type {
            type FnType = unsafe extern "system" fn($($arg_type),*) -> $ret_type;
            let original_fn: FnType = unsafe { crate::proxy::get_original_function($ordinal) };
            unsafe {original_fn($($arg_name),*)}
        }
    };
}
