use std::ffi::c_void;
use std::sync::{LazyLock, Mutex, Once};
use windows::core::{Interface, HRESULT};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct3D9::{
    IDirect3D9, IDirect3DDevice9, D3DPRESENT_PARAMETERS,
};
use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};

// --- Type definitions for the hooked functions ---
type FnPresent = unsafe extern "system" fn(
    this: IDirect3DDevice9,
    psourcerect: *const RECT,
    pdestrect: *const RECT,
    hdestwindowoverride: HWND,
    pdirtyregion: *const c_void,
) -> HRESULT;

type FnReset = unsafe extern "system" fn(
    this: IDirect3DDevice9,
    ppresentationparameters: *mut D3DPRESENT_PARAMETERS,
) -> HRESULT;

type FnCreateDevice = unsafe extern "system" fn(
    this: IDirect3D9,
    adapter: u32,
    devicetype: u32,
    hfocuswindow: HWND,
    behaviorflags: u32,
    ppresentationparameters: *mut D3DPRESENT_PARAMETERS,
    ppreturneddeviceinterface: *mut Option<IDirect3DDevice9>,
) -> HRESULT;

// --- Static storage for original function pointers ---
static mut O_CREATE_DEVICE: Option<FnCreateDevice> = None;
static mut O_RESET: Option<FnReset> = None;
static O_PRESENT: LazyLock<Mutex<Option<FnPresent>>> = LazyLock::new(|| Mutex::new(None));

// ===================================================================================
//  HOOK INSTALLATION
// ===================================================================================

/// Entry point for hooking. Called from `lib.rs` after `Direct3DCreate9` returns.
/// This function hooks ONLY `CreateDevice`.
pub unsafe fn install(d3d9: *mut IDirect3D9) { unsafe {
    let vtable_ptr = *(d3d9 as *mut *mut usize);
    let create_device_fn_ptr_location = vtable_ptr.add(16); // IDirect3D9::CreateDevice is at index 16

    // Save the original function pointer.
    O_CREATE_DEVICE = Some(std::mem::transmute(create_device_fn_ptr_location.read()));

    // Patch the VTable to point to our hooked function.
    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    VirtualProtect(
        create_device_fn_ptr_location as _,
        std::mem::size_of::<usize>(),
        PAGE_EXECUTE_READWRITE,
        &mut old_protect
    ).unwrap();

    create_device_fn_ptr_location.write(hooked_create_device as usize);

    VirtualProtect(
        create_device_fn_ptr_location as _,
        std::mem::size_of::<usize>(),
        old_protect,
        &mut old_protect
    ).unwrap();

    log::info!("[HOOK] IDirect3D9::CreateDevice hooked.");
}}


// ===================================================================================
//  HOOK IMPLEMENTATIONS
// ===================================================================================

/// Our hooked `CreateDevice`. Runs when the game creates a D3D device.
/// This function calls the original `CreateDevice` and then hooks `Present` and `Reset`.
unsafe extern "system" fn hooked_create_device(
    this: IDirect3D9,
    adapter: u32,
    devicetype: u32,
    hfocuswindow: HWND,
    behaviorflags: u32,
    ppresentationparameters: *mut D3DPRESENT_PARAMETERS,
    ppreturneddeviceinterface: *mut Option<IDirect3DDevice9>,
) -> HRESULT { unsafe {
    // First, let the game create the device by calling the original function.
    let result = O_CREATE_DEVICE.unwrap()(
        this,
        adapter,
        devicetype,
        hfocuswindow,
        behaviorflags,
        ppresentationparameters,
        ppreturneddeviceinterface,
    );

    // If the device was created successfully, we can now hook its VTable.
    if result.is_ok() {
        if let Some(Some(device)) = ppreturneddeviceinterface.as_mut() {
            crate::initialize_render(hfocuswindow, device);
            let device_vtable_ptr = *(device.as_raw() as *mut *mut usize);

            // --- Hook Present (index 17) ---
            let present_fn_ptr_location = device_vtable_ptr.add(17);
            let mut present_guard = O_PRESENT.lock().unwrap();
            if present_guard.is_none() {
                let original_present_addr = present_fn_ptr_location.read();
                *present_guard = Some(std::mem::transmute(original_present_addr));

                let mut old_protect = PAGE_PROTECTION_FLAGS(0);
                VirtualProtect(present_fn_ptr_location as _, std::mem::size_of::<usize>(), PAGE_EXECUTE_READWRITE, &mut old_protect).unwrap();
                present_fn_ptr_location.write(hooked_present as usize);
                VirtualProtect(present_fn_ptr_location as _, std::mem::size_of::<usize>(), old_protect, &mut old_protect).unwrap();
                log::info!("[HOOK] IDirect3DDevice9::Present hooked.");
            }

            // --- Hook Reset (index 16) ---
            let reset_fn_ptr_location = device_vtable_ptr.add(16);

            // SAFETY: This is only called once per device. The `O_RESET` static is only written to from here,
            // and this entire block is guarded by checking `is_none()`, ensuring that multiple threads
            // or multiple calls to `hooked_create_device` do not race to initialize `O_RESET`.
            #[allow(static_mut_refs)]
            if  O_RESET.is_none() {
                O_RESET = Some(std::mem::transmute(reset_fn_ptr_location.read()));

                let mut old_protect = PAGE_PROTECTION_FLAGS(0);
                VirtualProtect(reset_fn_ptr_location as _, std::mem::size_of::<usize>(), PAGE_EXECUTE_READWRITE, &mut old_protect).unwrap();
                reset_fn_ptr_location.write(hooked_reset as usize);
                VirtualProtect(reset_fn_ptr_location as _, std::mem::size_of::<usize>(), old_protect, &mut old_protect).unwrap();
                log::info!("[HOOK] IDirect3DDevice9::Reset hooked.");
            }
        }
    }
    result
}}

/// Our hooked `Reset`. Runs on resolution change, etc.
unsafe extern "system" fn hooked_reset(
    this: IDirect3DDevice9,
    ppresentationparameters: *mut D3DPRESENT_PARAMETERS,
) -> HRESULT {
    // Call pre_reset for egui
    crate::renderer::handle_pre_reset();

    // Call the original `Reset` function.
    let result = unsafe { O_RESET.unwrap()(this.clone(), ppresentationparameters) };
    if result.is_ok() {
        // Now we can call post_reset for egui
        crate::renderer::handle_post_reset(&this);
    }

    result
}


/// Our hooked `Present`. Runs every frame.
/// This is the main entry point for our mod's logic.
unsafe extern "system" fn hooked_present(
    this: IDirect3DDevice9,
    psourcerect: *const RECT,
    pdestrect: *const RECT,
    hdestwindowoverride: HWND,
    pdirtyregion: *const c_void,
) -> HRESULT {
    // One-time initialization for all our systems.
    static INIT_SYSTEMS_ONCE: Once = Once::new();
    INIT_SYSTEMS_ONCE.call_once(|| {
        crate::initialize_systems();
    });

    crate::renderer::render(&this);

    // Call the original `Present` function to let the game render.
    unsafe { O_PRESENT.lock().unwrap().unwrap()(
        this,
        psourcerect,
        pdestrect,
        hdestwindowoverride,
        pdirtyregion,
    ) }
}
