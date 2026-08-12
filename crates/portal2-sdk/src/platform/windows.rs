//! Windows implementation of the loader abstraction, on top of the Win32 API.

use std::ffi::{c_int, c_void, CStr};
use std::path::PathBuf;

use windows::core::PCSTR;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{
    GetModuleFileNameA, GetModuleHandleA, GetModuleHandleExA, GetProcAddress,
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};
use windows::Win32::System::ProcessStatus::{GetModuleInformation, MODULEINFO};
use windows::Win32::System::Threading::GetCurrentProcess;

use super::Module;

/// The `CreateInterface` export every Source module carries.
type CreateInterfaceFn =
    unsafe extern "C" fn(name: PCSTR, return_code: *mut c_int) -> *mut c_void;

/// Grabs a handle to an already-loaded module.
///
/// Deliberately `GetModuleHandleA` and not `LoadLibrary`: the engine owns these
/// modules, we only borrow them and must not touch their reference count.
fn module_handle(module: Module) -> Option<HMODULE> {
    let name = PCSTR(module.file_name().as_ptr() as *const u8);
    match unsafe { GetModuleHandleA(name) } {
        Ok(handle) if !handle.is_invalid() => Some(handle),
        _ => None,
    }
}

pub(super) fn module_range(module: Module) -> Option<(*const u8, usize)> {
    let handle = module_handle(module)?;

    let mut info = MODULEINFO::default();
    let ok = unsafe {
        GetModuleInformation(
            GetCurrentProcess(),
            handle,
            &mut info,
            size_of::<MODULEINFO>() as u32,
        )
        .is_ok()
    };

    if !ok {
        return None;
    }

    // SizeOfImage covers the whole mapped image. Unlike ELF, a PE image has no
    // PROT_NONE gaps between its sections, so the entire range is safe to read.
    Some((info.lpBaseOfDll as *const u8, info.SizeOfImage as usize))
}

pub(super) unsafe fn find_interface<T>(module: Module, interface_name: &CStr) -> *mut T {
    let Some(handle) = module_handle(module) else {
        log::warn!("Module is not loaded: {:?}", module);
        return std::ptr::null_mut();
    };

    let factory = unsafe { GetProcAddress(handle, PCSTR(c"CreateInterface".as_ptr() as *const u8)) };
    let Some(factory) = factory else {
        log::error!("'CreateInterface' not found in {:?}", module);
        return std::ptr::null_mut();
    };

    let factory: CreateInterfaceFn = unsafe { std::mem::transmute(factory) };
    unsafe { factory(PCSTR(interface_name.as_ptr() as *const u8), std::ptr::null_mut()) as *mut T }
}

pub(super) fn module_dir_of_self() -> Option<PathBuf> {
    unsafe {
        let mut handle = HMODULE::default();

        // Resolving by the address of this very function is what makes it "self",
        // regardless of whether we were injected as a proxy DLL or a plugin.
        GetModuleHandleExA(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            PCSTR(module_dir_of_self as *const u8),
            &mut handle,
        )
        .ok()?;

        let mut path = vec![0u8; 512];
        let len = GetModuleFileNameA(Some(handle), &mut path) as usize;
        if len == 0 {
            return None;
        }

        let path = std::str::from_utf8(&path[..len]).ok()?;
        PathBuf::from(path).parent().map(PathBuf::from)
    }
}
