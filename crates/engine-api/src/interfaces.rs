use std::ffi::{c_int, c_void};
use windows::core::PCSTR;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

/// A generic function pointer type for the `CreateInterface` export.
type CreateInterfaceFn =
    unsafe extern "C" fn(p_name: PCSTR, p_return_code: *mut c_int) -> *mut c_void;


/// Finds and returns a pointer to a game interface from a specified module.
///
/// This is the core function for acquiring engine interfaces like `IVEngineClient`,
/// `IClientEntityList`, etc. It dynamically loads the module, finds the `CreateInterface`
/// exported function, and then calls it to request the desired interface by name.
///
/// # Arguments
///
/// * `module_name` - The name of the DLL to search in (e.g., `b"engine.dll\0"`).
/// * `interface_name` - The versioned name of the interface to request (e.g., `b"VEngineClient015\0"`).
///
/// # Returns
///
/// * A mutable raw pointer to the requested interface.
/// * Returns a null pointer if the module or interface cannot be found.
///
/// # Safety
///
/// This function is highly unsafe because:
/// 1. It relies on `GetModuleHandleA` and `GetProcAddress`, which can fail and return null pointers.
/// 2. It transmutes a raw pointer into a function pointer (`CreateInterfaceFn`), which is undefined behavior
///    if the address is invalid or the function signature is incorrect.
/// 3. It deals with null-terminated C-strings.
/// The caller is responsible for ensuring that the returned pointer is valid before dereferencing it.
pub unsafe fn find_interface<T>(
    module_name: &'static [u8],
    interface_name: &'static [u8],
) -> *mut T {
    // Convert byte slices to PCSTR for Windows API calls
    let module_pcstr = PCSTR(module_name.as_ptr());
    let interface_pcstr = PCSTR(interface_name.as_ptr());

    // Get a handle to the module (DLL) that is already loaded in the game's process.
    // This is safer than LoadLibrary as it doesn't increment the module's reference count.
    let module_handle = match unsafe { GetModuleHandleA(module_pcstr) } {
        Ok(handle) if !handle.is_invalid() => handle,
        _ => {
            // This is a critical error. The module should already be loaded by the game.
            log::error!("Failed to get module handle for: {}\0", String::from_utf8_lossy(module_name));
            return std::ptr::null_mut();
        }
    };

    // Find the exported `CreateInterface` function within the module.
    let create_interface_addr =
        match unsafe { GetProcAddress(module_handle, PCSTR(b"CreateInterface\0".as_ptr())) } {
            Some(addr) => addr,
            None => {
                log::error!("'CreateInterface' not found in: {}", String::from_utf8_lossy(module_name));
                return std::ptr::null_mut();
            }
        };

    // Cast the function address to the correct function pointer type.
    let create_interface: CreateInterfaceFn = unsafe { std::mem::transmute::<_, CreateInterfaceFn>(create_interface_addr) };

    // Call `CreateInterface` to get a pointer to the requested interface.
    // The second argument (return code) is optional and can be null.
    let interface_ptr = unsafe { create_interface(interface_pcstr, std::ptr::null_mut()) };

    // Return the pointer, casting it to the generic type `T`.
    // The caller will then cast it to a specific interface struct like `IVEngineClient`.
    interface_ptr as *mut T
}
